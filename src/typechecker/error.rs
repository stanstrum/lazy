use snafu::prelude::*;

use crate::tokenizer::{
  GetSpan,
  Span,
};

use crate::CompilationError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum TypeCheckerError {
  #[snafu(display("unknown variable: \"{name}\""))]
  UnknownVariable { name: String, span: Span },

  #[snafu(display("incompatible types: {lhs} and {rhs}"))]
  IncompatibleTypes { lhs: String, rhs: String, span: Span },
}

impl GetSpan for TypeCheckerError {
  fn get_span(&self) -> Span {
    match self {
      | TypeCheckerError::UnknownVariable { span, .. }
      | TypeCheckerError::IncompatibleTypes { span, .. } => *span,
    }
  }
}

impl From<TypeCheckerError> for CompilationError {
  fn from(error: TypeCheckerError) -> Self {
    Self::TypeCheck { error }
  }
}
