use snafu::prelude::*;

use crate::CompilationError;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum AsterizerError {
  #[snafu(display("Not implemented: {message}"))]
  NotImplemented { message: String },

  #[snafu(display("Expected {what}"))]
  Expected { what: String }
}

impl From<AsterizerError> for CompilationError {
  fn from(error: AsterizerError) -> Self {
    Self::Asterization { error }
  }
}
