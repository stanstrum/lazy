use std::marker::PhantomData;

use crate::compiler::{
  CompilerWorkflow,
  CompilerStoreHandle,
};

/// The debug information for a language object
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Span<W: CompilerWorkflow> {
  pub start: usize,
  pub end: usize,
  pub handle: CompilerStoreHandle<W>,
  marker: PhantomData<W>,
}

/// The beginning of a Span
#[derive(Debug, Clone, Copy)]
pub(crate) struct SpanStart<W: CompilerWorkflow> {
  pub start: usize,
  pub handle: CompilerStoreHandle<W>,
  pub marker: PhantomData<W>,
}

impl<W: CompilerWorkflow> Span<W> {
  /// Makes a SpanStart from the data in Self
  #[allow(unused)]
  pub(crate) fn as_start(&self) -> SpanStart<W> {
    SpanStart {
      start: self.start,
      handle: self.handle,
      marker: Default::default(),
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
      marker: Default::default(),
    }
  }
}
