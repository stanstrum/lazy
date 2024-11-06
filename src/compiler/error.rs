use snafu::prelude::*;

use utf8_read::Char;

use super::*;

use crate::{
  arg_parser::error::ArgumentError,
  asterizer::error::AsterizerError,
  tokenizer::error::TokenError,
  checker::error::CheckerError,
  Result,
};

use crate::tokenizer::Span;

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

impl From<ArgumentError> for CompilerError {
  fn from(err: ArgumentError) -> Self {
    Self::Argument { err }
  }
}

impl From<TokenError> for CompilerError {
  fn from(err: TokenError) -> Self {
    Self::Token { err }
  }
}

impl From<AsterizerError> for CompilerError {
  fn from(err: AsterizerError) -> Self {
    Self::Ast { err }
  }
}

impl From<CheckerError> for CompilerError {
  fn from(err: CheckerError) -> Self {
    Self::Check { err }
  }
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

    // Manually seek to the beginning of `span`
    for _ in 0..span.start {
      match reader.next_char() {
        Ok(Char::Char('\n')) => {
          line += 1;
          column = 1;
        },
        Ok(Char::Char(_)) => column += 1,
        Ok(Char::Eof) => return IOSnafu { err: "span starts outside of the end of the file" }.fail()?,
        Ok(Char::NoData) => return IOSnafu { err: "invalid UTF-8 in file" }.fail()?,
        Err(err) => return IOSnafu { err: err.to_string() }.fail()?,
      };
    };

    // Calculate the length of `span` -- this may be zero but never negative
    let length = span.end - span.start;

    // Now read the section of code we're actually looking for
    let text = match reader.take(length).collect() {
      Ok(x) => x,
      // TODO: make this an implicit Into -- getting annoying
      Err(err) => return IOSnafu { err: err.to_string() }.fail()?,
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
