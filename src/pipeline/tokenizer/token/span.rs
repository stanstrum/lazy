use crate::compiler::{CompilerStoreHandle, CompilerWorkflow};

/// The debug information for a language object
#[allow(unused)]
#[derive(Clone, Copy)]
pub(crate) struct Span<W: CompilerWorkflow> {
  pub start: usize,
  pub end: usize,
  pub handle: CompilerStoreHandle<W>,
}

/// The beginning of a Span
#[derive(Debug, Clone, Copy)]
pub(crate) struct SpanStart<W: CompilerWorkflow> {
  pub start: usize,
  pub handle: CompilerStoreHandle<W>,
}

impl<W: CompilerWorkflow> Span<W> {
  /// Makes a SpanStart from the data in Self
  #[allow(unused)]
  pub(crate) fn as_start(&self) -> SpanStart<W> {
    SpanStart {
      start: self.start,
      handle: self.handle,
    }
  }
}

impl<W: CompilerWorkflow> SpanStart<W> {
  /// Make a Span from Self and end
  pub(crate) fn into_span(self, end: usize) -> Span<W> {
    Span {
      start: self.start,
      end,
      handle: self.handle,
    }
  }
}

impl<W: CompilerWorkflow + std::fmt::Debug> std::fmt::Debug for Span<W> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!(
      "Span({}, {}, {:?})",
      self.start, self.end, &self.handle
    ))
  }
}
