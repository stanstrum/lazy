/// The debug information for a language object
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Span {
  pub start: usize,
  pub end: usize,
}

/// The beginning of a Span
#[derive(Debug, Clone, Copy)]
pub(crate) struct SpanStart(pub usize);

impl Span {
  /// Makes a SpanStart from the data in Self
  pub(crate) fn into_start(&self) -> SpanStart {
    SpanStart(self.start)
  }
}

impl SpanStart {
  /// Make a Span from Self and end
  pub(crate) fn into_span(&self, end: usize) -> Span {
    Span {
      start: self.0,
      end,
    }
  }
}
