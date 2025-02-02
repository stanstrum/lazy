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

  fn alloc_index(&mut self, index: usize) {
    if index >= self.buffer.len() {
      for _ in 0..=(index - self.buffer.len()) {
        let Some(token) = self.iter.next() else {
          return;
        };

        // important! set mark so we know where we are when crashing out
        self.mark = token.1.start;

        self.buffer.push(token);
      }
    };
  }

  fn seek_to(&mut self, index: usize) {
    self.alloc_index(index);
    self.index = index;
  }

  pub(super) fn seek(&mut self) {
    self.seek_to(self.index + 1);
  }

  fn rewind(&mut self) {
    self.index -= 1;
  }

  pub(crate) fn peek(&mut self) -> Option<&Token> {
    self.alloc_index(self.index);
    self.buffer.get(self.index)
  }

  pub(super) fn next(&mut self) -> Option<&Token> {
    self.seek_to(self.index + 1);
    self.buffer.get(self.index - 1)
  }

  pub(super) fn pop(&mut self, mark: SourceMark) {
    while self.index > 0 {
      'attempt: {
        let Some((_, Span { start, .. })) = self.peek() else {
          break 'attempt;
        };

        assert!(mark.start_of_line_byte >= start.start_of_line_byte);
        assert!(mark.column >= start.column);

        if (mark.start_of_line_byte, mark.column) == (start.start_of_line_byte, start.column) {
          return;
        };

        // otherwise ...
      };

      self.rewind();
    }
  }
}
