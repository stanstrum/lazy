use snafu::prelude::*;

use crate::CompilationError;

use super::{
  Span,
  GetSpan,
  Token,
};

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum TokenizationError {
  IOError { error: utf8_read::Error },

  #[snafu(display("failed to parse source"))]
  InvalidSource {
    parsed: Vec<Token>,
    #[snafu(source(false))]
    source: String,
    span: Span
  }
}

impl From<utf8_read::Error> for TokenizationError {
  fn from(error: utf8_read::Error) -> Self {
    Self::IOError { error }
  }
}

impl From<TokenizationError> for CompilationError {
  fn from(error: TokenizationError) -> Self {
    Self::Tokenization { error }
  }
}

impl GetSpan for TokenizationError {
  fn get_span(&self) -> Span {
    let Self::InvalidSource { span, .. } = self else {
      panic!("attempted to get span of a non-logic-based tokenization error");
    };

    *span
  }
}
