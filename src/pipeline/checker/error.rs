use snafu::prelude::*;

use crate::compiler::error::{CompilerError, ReadSpan};

#[derive(Debug)]
pub(crate) struct ReprSpan {
  pub(crate) repr: String,
  pub(crate) span: ReadSpan,
}

#[allow(unused)]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum CheckerError {
  #[snafu(display("unresolved identifier:"))]
  UnresolvedQualified { span: ReadSpan },

  #[snafu(display("type mismatch:"))]
  TypeMismatch {
    span: ReadSpan,
    found_repr: String,
    expected_repr: String,
  },

  #[snafu(display("unresolved literal:"))]
  UnresolvedLiteral {
    span: ReadSpan,
  },
}

impl CheckerError {
  pub(crate) fn applicable_span(self) -> Option<ReadSpan> {
    match self {
      | CheckerError::UnresolvedQualified { span, .. }
      | CheckerError::UnresolvedLiteral { span }
      | CheckerError::TypeMismatch { span, .. } => Some(span),
    }
  }
}

impl From<CheckerError> for CompilerError {
  fn from(err: CheckerError) -> Self {
    Self::Check { err }
  }
}
