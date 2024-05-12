use snafu::prelude::*;

use crate::CompilationError;

use crate::tokenizer::Span;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum AsterizerError {
  #[snafu(display("Not implemented: {message}"))]
  NotImplemented { message: String, span: Span },

  #[snafu(display("Expected {what}"))]
  Expected { what: String, span: Span }
}

impl From<AsterizerError> for CompilationError {
  fn from(error: AsterizerError) -> Self {
    Self::Asterization { error }
  }
}
