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
  UnknownVariable { name: String }
}

impl GetSpan for TypeCheckerError {
  fn get_span(&self) -> &Span {
    match self {
      _ => todo!("type check span: {self:?}")
    }
  }
}

impl From<TypeCheckerError> for CompilationError {
  fn from(error: TypeCheckerError) -> Self {
    Self::TypeCheck { error }
  }
}
