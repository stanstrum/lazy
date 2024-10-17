use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum ArgumentError {
  Help,

  #[snafu(display("unrecognized flag: {flag}"))]
  UnknownFlag { flag: String },

  #[snafu(display("already received argument: {long_name}"))]
  Duplicate { long_name: String },

  #[snafu(display("could not find executable: {path}\n{err}"))]
  ExecNotFound { path: String, err: which::Error },

  #[snafu(display("no input file provided"))]
  NoInput,
}

impl crate::LazyHelp for ArgumentError {
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
