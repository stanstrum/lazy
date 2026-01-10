#[derive(Debug)]
pub struct Metadata {
  pub line: usize,
  pub column: usize,
  pub whitespace: usize,
  reading_whitespace: bool,
  last_ch: Option<char>,
}

impl Metadata {
  pub(super) fn new() -> Self {
    Self {
      line: 1,
      column: 1,
      whitespace: 0,
      reading_whitespace: true,
      last_ch: None,
    }
  }

  pub(super) fn take(&mut self, ch: char) {
    let Some(ch) = self.last_ch.replace(ch) else {
      return;
    };

    if ch == '\n' {
      self.line += 1;
      self.column = 1;
      self.reading_whitespace = true;

      return;
    };

    if !self.reading_whitespace {
      self.column += 1;

      return;
    };

    if matches!(ch, ' ' | '\t') {
      if self.column == 1 {
        self.whitespace = 1;
      } else {
        self.whitespace += 1;
      };

      self.column += 1;

      return;
    };

    if self.column == 1 {
      self.whitespace = 0;
    };

    self.column += 1;
    self.reading_whitespace = false;
  }
}
