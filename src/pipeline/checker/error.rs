use snafu::prelude::*;

use crate::compiler::error::ReadSpan;

#[allow(unused)]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum CheckerError {
  #[snafu(display("unresolved identifier:"))]
  UnresolvedQualified {
    span: ReadSpan,
  },
}

impl CheckerError {
  pub(crate) fn applicable_span(self) -> Option<ReadSpan> {
    match self {
      CheckerError::UnresolvedQualified { span,  ..  } => Some(span),
    }
  }
}
