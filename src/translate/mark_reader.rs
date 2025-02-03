use super::*;

impl<I: Iterator<Item = Token>> Translator<I> {
  pub(crate) fn new(id: usize, iter: I, tx: &Sender<CompilerSignal>) -> Self {
    Self {
      id,
      iter,
      index: 0,
      buffer: vec![],
      tx: tx.clone(),
      mark: SourceMark::default(),
    }
  }

  pub(crate) fn seek(&mut self) {
    println!("reader @ {}: seek", self.index);
    self.index += 1;
  }

  fn rewind(&mut self) {
    self.index -= 1;
  }

  fn get_nth(&mut self, index: usize) -> Option<&Token> {
    if index >= self.buffer.len() {
      for _ in 0..=(index - self.buffer.len()) {
        let token = self.iter.next()?;
        self.buffer.push(token);
      }
    }

    self.buffer.get(index)
  }

  pub(crate) fn peek(&mut self) -> Option<&Token> {
    self.get_nth(self.index)
  }

  pub(crate) fn next(&mut self) -> Option<&Token> {
    self.seek();
    self.get_nth(self.index - 1)
  }

  pub(super) fn pop(&mut self, mark: SourceMark) {
    while self.index > 0 {
      let Some((_, Span { start, .. })) = self.peek() else {
        self.rewind();
        continue;
      };

      assert!(start.start_of_line_byte >= mark.start_of_line_byte);
      assert!(start.column >= mark.column);

      if (start.start_of_line_byte, start.column) == (mark.start_of_line_byte, start.column) {
        return;
      };

      // otherwise
      self.rewind();
    }
  }
}
