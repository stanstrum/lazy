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
