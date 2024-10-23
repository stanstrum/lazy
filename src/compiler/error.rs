use snafu::prelude::*;

use std::path::PathBuf;

use crate::arg_parser::error::ArgumentError;
use crate::asterizer::errors::AsterizerError;
use crate::tokenizer::error::TokenError;

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
  #[snafu(display("Token error: {err:?}"))]
  Token { err: TokenError },

  /// An error occurred when asterizing a file's source code
  #[snafu(display("AST error: {err:?}"))]
  Ast { err: AsterizerError },
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
}
