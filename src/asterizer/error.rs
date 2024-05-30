use snafu::prelude::*;

use crate::CompilationError;

use crate::tokenizer::{
  Span,
  GetSpan,
};

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum AsterizerError {
  #[snafu(display("not implemented: {message}"))]
  NotImplemented { message: String, span: Span },

  #[snafu(display("expected {what}"))]
  Expected { what: String, span: Span }
}

impl GetSpan for AsterizerError {
  fn get_span(&self) -> &Span {
    match &self {
      AsterizerError::NotImplemented { span, .. } => span,
      AsterizerError::Expected { span, .. } => span,
    }
  }
}

impl From<AsterizerError> for CompilationError {
  fn from(error: AsterizerError) -> Self {
    Self::Asterization { error }
  }
}
