use snafu::prelude::*;

use crate::tokenizer::GetSpan;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum GeneratorError {
}

impl GetSpan for GeneratorError {
  fn get_span(&self) -> crate::tokenizer::Span {
    todo!()
  }
}
