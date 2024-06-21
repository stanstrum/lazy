use snafu::prelude::*;

use crate::tokenizer::{
  GetSpan,
  Span,
};

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
