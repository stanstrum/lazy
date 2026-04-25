use crate::Compiler;

pub trait GetSpan<C: Compiler> {
  fn get_span(&self, store: &C::Store<'_>) -> Span<C>;
}

// impl<'a, C: Compiler, R: Reference<C::Store<'a>>> GetSpan<C> for R
//   where <C as Compiler>::Store<'a>: Store<R>,
//         <<C as Compiler>::Store<'a> as Store<R>>::Out: GetSpan<C>
// {
//   fn get_span(&self, store: &<C as Compiler>::Store<'a>) -> Span<C> {
//     self.rget_from(store).get_span(store)
//   }
// }

/// Contains only the start position of a Span
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
  pub position: usize,
  pub line: usize,
  pub column: usize,
  pub indentation: usize,
}

impl Default for Position {
  fn default() -> Self {
    Self {
      position: 0,
      line: 1,
      column: 1,
      indentation: 0,
    }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Span<C: Compiler> {
  pub start: Position,
  pub end: Position,
  pub module: C::ModuleReference,
}

impl<C: Compiler> Span<C> {
  pub fn from_pair(start: Self, end: Self) -> Self {
    assert!(start.module == end.module,
      "from_pair requires the pair of spans be from the same file"
    );

    Self {
      start: start.start,
      end: end.end,
      module: start.module,
     }
  }

  pub fn extend(&mut self, other: Self) {
    assert!(self.module == other.module);
    self.end = other.end;
  }
}
