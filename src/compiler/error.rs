use snafu::prelude::*;

use std::path::PathBuf;

use crate::arg_parser::error::ArgumentError;
use crate::tokenizer::error::TokenError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum CompilerError {
  #[snafu(display("IO error: {err}"))]
  IO { err: String },

  #[snafu(display("file does not exist: {}", path.to_string_lossy()))]
  PathNotExists { path: PathBuf },

  #[snafu(display("path is a directory: {}", path.to_string_lossy()))]
  PathIsDirectory { path: PathBuf },

  #[snafu(display("{err}"))]
  Argument { err: ArgumentError },

  #[snafu(display("Token error: {err:?}"))]
  Token { err: TokenError },
}

impl From<TokenError> for CompilerError {
  fn from(err: TokenError) -> Self {
    Self::Token { err }
  }
}

impl From<ArgumentError> for CompilerError {
  fn from(err: ArgumentError) -> Self {
    Self::Argument { err }
  }
}

impl crate::LazyHelp for CompilerError {
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
