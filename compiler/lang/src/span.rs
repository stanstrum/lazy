use crate::Compiler;

/// Contains only the start position of a Span
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
  pub position: usize,
  pub line: usize,
  pub column: usize,
  pub indentation: usize,
}

impl Position {
  pub fn new() -> Self {
    Self {
      position: 0,
      line: 1,
      column: 1,
      indentation: 0,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Span<C: Compiler> {
  pub start: Position,
  pub end: Position,
  pub module: C::ModuleReference,
}

impl<C: Compiler + Copy> Copy for Span<C>
where C::ModuleReference: Copy
{
}

impl<C: Compiler + PartialEq> PartialEq for Span<C>
  where C::ModuleReference: PartialEq
{
  fn eq(&self, other: &Self) -> bool {
    self.start == other.start && self.end == other.end && self.module == other.module
  }
}

impl<C: Compiler + Eq> Eq for Span<C> where C::ModuleReference: Eq {}

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
