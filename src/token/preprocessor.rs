use super::*;

impl<T: Iterator<Item = (char, usize)>> Tokenizer<T> {
  pub(super) fn read_ch(&mut self) -> Option<char> {
    if let Some(ch) = self.tail_call.take() {
      return Some(ch);
    };

    let (ch, position) = self.iter.next()?;

    if self.start_of_line {
      self.mark.line += 1;
      self.mark.column = 0;
      self.mark.start_of_line_byte = position;
      self.start_of_line = false;
    };

    if self.indent_buf.len() > self.mark.column {
      if self.indent_buf.chars().nth(self.mark.column).unwrap() != ch {
        println!(
          "read_within_line: indent -= {}",
          self.indent_buf.len() - self.mark.column
        );
        self.indent_buf.drain(self.mark.column..);
      };
    } else if self.indent_buf.len() == self.mark.column && matches!(ch, ' ' | '\t') {
      self.indent_buf.push(ch);
    };

    self.mark.column += 1;

    if ch == '\n' {
      self.start_of_line = true;
    };

    Some(ch)
  }

  pub(super) fn indentation(&self) -> usize {
    self.indent_buf.len()
  }

  pub(super) fn push_token(&mut self, kind: TokenKind, start: SourceMark) {
    let token = (
      kind,
      Span {
        id: self.id,
        start,
        end: self.mark,
      },
    );

    self.tokens.push_back(dbg!(token));
  }
}

impl<T: Iterator<Item = (char, usize)>> Iterator for Tokenizer<T> {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    if self.tokens.is_empty() {
      self.base();
    };

    self.tokens.pop_front()
  }
}
