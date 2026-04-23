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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleSpan<M> {
  pub start: Position,
  pub end: Position,
  pub module: M,
}

impl<M: PartialEq> ModuleSpan<M> {
  pub fn from_pair(start: ModuleSpan<M>, end: ModuleSpan<M>) -> Self {
    assert!(start.module == end.module,
      "from_pair requires the pair of spans be from the same file"
    );

    Self {
      start: start.start,
      end: end.end,
      module: start.module,
    }
  }

  pub fn extend(&mut self, other: ModuleSpan<M>) {
    assert!(self.module == other.module);
    self.end = other.end;
  }
}
