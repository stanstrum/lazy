use snafu::prelude::*;

use crate::tokenizer::{
  GetSpan,
  Span,
};

use crate::CompilationError;

#[derive(Debug, Snafu)]
pub(crate) enum TypeCheckerError {
}

impl GetSpan for TypeCheckerError {
  fn get_span(&self) -> &Span {
    match self {
      _ => todo!()
    }
  }
}

impl From<TypeCheckerError> for CompilationError {
  fn from(error: TypeCheckerError) -> Self {
    Self::TypeCheck { error }
  }
}
