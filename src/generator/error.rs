use snafu::prelude::*;

use crate::{tokenizer::{GetSpan, Span}, CompilationError};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum GeneratorError {
  #[snafu(display("unresolved type"))]
  Unresolved { span: Span }
}

impl GetSpan for GeneratorError {
  fn get_span(&self) -> crate::tokenizer::Span {
    match self {
      GeneratorError::Unresolved { span } => *span,
    }
  }
}

impl From<GeneratorError> for CompilationError {
  fn from(error: GeneratorError) -> Self {
    Self::Generation { error }
  }
}
