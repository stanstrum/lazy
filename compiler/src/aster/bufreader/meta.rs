#[derive(Debug)]
pub struct Metadata {
  pub line: usize,
  pub column: usize,
  pub whitespace: usize,
  pub position: usize,
  pub reading_whitespace: bool,
  last_ch: Option<char>,
}

impl Metadata {
  pub(super) fn new() -> Self {
    Self {
      line: 1,
      column: 1,
      whitespace: 0,
      position: 0,
      reading_whitespace: true,
      last_ch: None,
    }
  }

  pub(super) fn take(&mut self, ch: char) {
    let Some(ch) = self.last_ch.replace(ch) else {
      return;
    };

    // This is the _byte_ position, not character count
    self.position += ch.len_utf8();

    if ch == '\n' {
      self.line += 1;
      self.column = 1;

      self.whitespace = 0;
      self.reading_whitespace = true;

      return;
    };

    if self.reading_whitespace {
      if matches!(ch, ' ' | '\t') {
        self.whitespace += 1;
      } else {
        self.reading_whitespace = false;
      };
    };

    self.column += 1;
  }
}
