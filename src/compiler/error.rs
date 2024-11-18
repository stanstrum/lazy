use snafu::prelude::*;
use utf8_read::Char;

use super::*;
use crate::tokenizer::Span;
use crate::{
  arg_parser::error::ArgumentError, asterizer::error::AsterizerError, checker::error::CheckerError,
  tokenizer::error::TokenError, Result,
};
use crate::help::LazyHelp;

#[allow(unused)]
/// Stores error information taken from a Span without the need for propagating
/// the W: CompilerWorkflow constraint to bearers down the error-handling path
#[derive(Debug)]
pub(crate) struct ReadSpan {
  pub(crate) path: CompilerModulePath,
  pub(crate) start: usize,
  pub(crate) end: usize,
  pub(crate) line: usize,
  pub(crate) column: usize,
  pub(crate) text: String,
}

/// Represents an error encounted at any point during the compilation process
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum CompilerError {
  /// An IO error occured
  #[snafu(display("IO error: {err}"))]
  IO { err: String },

  /// A required path was found to not exist
  #[snafu(display("file does not exist: {}", path.to_string_lossy()))]
  PathNotExists { path: PathBuf },

  /// A required path was found to be a directory rather than a file
  #[snafu(display("path is a directory: {}", path.to_string_lossy()))]
  PathIsDirectory { path: PathBuf },

  /// An error occurred when parsing command-line arguments
  #[snafu(display("{err}"))]
  Argument { err: ArgumentError },

  /// An error occurred when tokenizing a file's source code
  #[snafu(display("Token error: {err}"))]
  Token { err: TokenError },

  /// An error occurred when asterizing a file's source code
  #[snafu(display("AST error: {err}"))]
  Ast { err: AsterizerError },

  /// An error occurred when checking a file's source tree
  #[snafu(display("type check error: {err}"))]
  Check { err: CheckerError },
}

impl crate::help::LazyHelp for CompilerError {
  fn should_print_message(&self) -> bool {
    match self {
      CompilerError::Argument { err } => err.should_print_message(),
      _ => true,
    }
  }

  fn should_print_help_text(&self) -> bool {
    match self {
      CompilerError::Argument { err } => err.should_print_help_text(),
      _ => false,
    }
  }

  fn applicable_span(self) -> Option<ReadSpan> {
    match self {
      CompilerError::Ast { err } => err.applicable_span(),
      CompilerError::Check { err } => err.applicable_span(),
      _ => None,
    }
  }
}

trait CatchStreamError {
  fn catch_stream_error(&self) -> Result<char>;
}

impl CatchStreamError for Char {
  fn catch_stream_error(&self) -> Result<char> {
    match self {
      Char::Char(ch) => Ok(*ch),
      Char::Eof => {
        IOSnafu {
          err: "span exists outside of the end of the file",
        }
        .fail()?
      },
      Char::NoData => {
        IOSnafu {
          err: "invalid UTF-8 in file",
        }
        .fail()?
      },
    }
  }
}

impl CompilerError {
  pub(crate) fn output_to_logger(self) {
    // Decide how to present the error to the user; e.g.:
    // the help flag should print the help text
    let should_print_help_text = self.should_print_help_text();
    let should_print_message = self.should_print_message();

    if should_print_help_text {
      crate::help::print_help_text();

      // Put a space between the help text and the error message for clarity
      if should_print_message {
        eprintln!();
      };
    };

    if should_print_message {
      crate::help::print_message(self);
    };
  }
}

impl<W: CompilerWorkflow> Compiler<W> {
  pub(crate) fn span_to_read_span(&self, span: Span<W>) -> Result<ReadSpan> {
    // Get module by the handle provided by `span`
    let module = self.store.get_module(&span.handle);
    // Get the module's path
    let path = &module.path;

    // Open said file
    let file = path.bytes()?;

    // TODO: low-hanging fruit (see below):
    // Create a UTF-8 reader ... again.  This is because Spans currently hold
    // position counters that are based in UTF-8 characters and not bytes,
    // therefore we can't simply seek to the specified position of the error.
    // This can likely be easily fixed by rewriting Tokenizer code so that the
    // length of the bytes is preserved for this specific situation.  However
    // for now, we'll just do the hard work.
    let mut reader = utf8_read::Reader::new(file);

    // As a result of having to manually Seek the file, we can actually find out
    // at this point the line and column that the error is found on.
    // TODO: this feels like the wrong place to determine this.  Maybe Spans
    // should include this information as well?

    // These are solely for the end-user to see, so we'll use human-readable/
    // one-indexed counters
    let mut line = 1;
    let mut column = 1;

    let mut current_line = "1: ".to_string();

    // Manually seek to the beginning of `span`
    for _ in 0..span.start {
      match reader.next_char()?.catch_stream_error()? {
        '\n' => {
          line += 1;
          column = 1;
          current_line = format!("{line}: ");
        },
        ch => {
          current_line.push(ch);
          column += 1;
        },
      };
    };

    let mut text = current_line;

    // Now seek to the end of the selection, since we need to find the end of
    // that line
    for _ in span.start..span.end {
      let ch = reader.next_char()?.catch_stream_error()?;
      text.push(ch);

      if ch == '\n' {
        line += 1;
        text += format!("{line}: ").as_str();
      };
    };

    loop {
      match reader.next_char()? {
        err @ Char::NoData => { err.catch_stream_error()?; },
        | Char::Char('\n')
        | Char::Eof => break,
        Char::Char(ch) => { text.push(ch); },
      };
    };

    Ok(ReadSpan {
      path: path.to_owned(),
      start: span.start,
      end: span.end,
      line,
      column,
      text,
    })
  }
}

impl From<utf8_read::Error> for CompilerError {
  fn from(err: utf8_read::Error) -> Self {
    Self::IO { err: err.to_string() }
  }
}
