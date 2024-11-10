use snafu::prelude::*;

use super::CompilerError;

/// An error encountered when parsing command-line arguments
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum ArgumentError {
  /// Returned when the help flag is returned; no error message is printed, but
  /// the help text will be displayed and the program will exit immediately with
  /// a non-zero error code
  Help,

  /// An argument was provided that could not be parsed
  #[snafu(display("unrecognized flag: {flag}"))]
  UnknownFlag { flag: String },

  /// An argument was provided more than once
  #[snafu(display("already received argument: {long_name}"))]
  Duplicate { long_name: String },

  /// A required executable could not be located
  #[snafu(display("could not find executable: {path}\n{err}"))]
  ExecNotFound { path: String, err: which::Error },

  /// No input file was provided
  #[snafu(display("no input file provided"))]
  NoInput,
}

impl crate::help::LazyHelp for ArgumentError {
  fn should_print_message(&self) -> bool {
    !matches!(self, ArgumentError::Help)
  }

  fn should_print_help_text(&self) -> bool {
    matches!(self,
      | ArgumentError::Help
      | ArgumentError::UnknownFlag { .. }
    )
  }
}

impl From<ArgumentError> for CompilerError {
  fn from(err: ArgumentError) -> Self {
    Self::Argument { err }
  }
}
