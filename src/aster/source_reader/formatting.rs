use super::{SourceReader, Span};

fn num_length<T: std::convert::Into<f64>>(n: T) -> usize {
  f64::from(
    (n.into().log10() + 1.0).floor().max(1.0)
  ) as usize
}

pub enum Level {
  Debug,
  Note,
  Warning,
  Error
}

impl std::string::ToString for Level {
  fn to_string(&self) -> String {
    match self {
      Level::Debug => {
        concat!("\x1b[1;35m", "debug", "\x1b[0m")
      },
      Level::Note => {
        concat!("\x1b[1;36m", "note", "\x1b[0m")
      },
      Level::Warning => {
        concat!("\x1b[1;33m", "warning", "\x1b[0m")
      },
      Level::Error => {
        concat!("\x1b[1;31m", "error", "\x1b[0m")
      },
    }.to_string()
  }
}

struct Message {
  level: Level,
  msg: String,
  span: Span,
}

impl SourceReader<'_> {
  pub fn span_since(&self, start: usize) -> Span {
    Span { start, end: self.offset }
  }

  pub fn at(&self) -> String {
    // seek to beginning of line
    let mut start = self.offset;
    let mut end = self.offset;

    loop {
      // println!("start: {}:{:#?}", start, self.src.chars().nth(start).unwrap());

      if start == 0 {
        break;
      };

      if self.src.chars().nth(start).unwrap() == '\n' {
        break;
      };

      start -= 1;
    };

    loop {
      // println!("end: {}:{:#?}", end, self.src.chars().nth(end).unwrap());

      if end == self.len() {
        break;
      };

      if self.src.chars().nth(end).unwrap() == '\n' {
        break;
      };

      end += 1;
    };

    if start + 1 == end {
      panic!("SourceReader::at called on an empty line");
    }

    if self.src.chars().nth(start).unwrap() == '\n' {
      start += 1;
    };

    if self.src.chars().nth(end).unwrap() == '\n' {
      end -= 1;
    };


    if start == self.src.len() {
      dbg!(self.offset);
      return "<end of file>".to_string();
    };

    // println!("end: {}:{:#?}", end, self.src.chars().nth(end).unwrap());
    // println!("start: {}:{:#?}", start, self.src.chars().nth(start).unwrap());

    let mut line: usize = 0;
    for ch in self.src[..start].chars() {
      if ch == '\n' {
        // println!("line {}:{:#?}", line, ch);

        line += 1;
      };
    };

    let col = self.offset - start;
    // println!("offset: {offset}, start: {start}, end: {end}, line: {line}, col: {col}", offset=self.offset());
    // println!("on: {:#?}", self.src.chars().nth(start).unwrap());

    let line_no_length = num_length(line as u32);
    let col_length = num_length(col as u32);

    if start != end && self.src.chars().nth(start).unwrap() == '\n' {
      start += 1;
    };

    format!(
      "{}:{}: {}\n{}^ here",
      line + 1, col + 1,
      &self.src[start..=end],
      "~".repeat(
        col + line_no_length + col_length
        + 3 // length of ":" and ": "
      )
    )
  }
}
