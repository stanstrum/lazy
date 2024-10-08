#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Span {
  start: usize,
  end: usize,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::tokenizer) struct SpanStart(pub usize);

impl SpanStart {
  pub(in crate::tokenizer) fn into_span(&self, end: usize) -> Span {
    Span {
      start: self.0,
      end,
    }
  }
}

