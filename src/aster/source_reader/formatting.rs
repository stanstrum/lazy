use super::{SourceReader, Span};
use std::io::Write;

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

pub struct Message {
  pub level: Level,
  pub msg: String,
  pub span: Span,
}

pub fn start_end(src: &String, offset: usize) -> Option<(usize, usize)> {
  let mut start = offset;
  let mut end = offset;

  loop {
    // println!("start: {}:{:#?}", start, &self.src.chars().nth(start).unwrap());

    if start == 0 {
      break;
    };

    if src.chars().nth(start).unwrap() == '\n' {
      break;
    };

    start -= 1;
  };

  loop {
    // println!("end: {}:{:#?}", end, &self.src.chars().nth(end).unwrap());

    if end == src.len() {
      break;
    };

    if src.chars().nth(end).unwrap() == '\n' {
      break;
    };

    end += 1;
  };

  if start + 1 == end {
    panic!("SourceReader::at called on an empty line");
  }

  if src.chars().nth(start).unwrap() == '\n' {
    start += 1;
  };

  if src.chars().nth(end).unwrap() == '\n' {
    end -= 1;
  };

  if start == src.len() {
    return None;
  };

  if start != end && src.chars().nth(start).unwrap() == '\n' {
    start += 1;
  };

  Some((start, end))
}

pub fn line_col(src: &String, offset: usize) -> (usize, usize) {
  let (start, _) = start_end(src, offset).unwrap();

  let mut line: usize = 0;
  for ch in src[..start].chars() {
    if ch == '\n' {
      // println!("line {}:{:#?}", line, ch);

      line += 1;
    };
  };

  let col = offset - start;

  (line, col)
}

fn at(src: &String, offset: usize) -> String {
  // seek to beginning of line
  let Some((start, end)) = start_end(src, offset) else {
    return "<failed getting start, end from offset>".to_string();
  };

  // println!("end: {}:{:#?}", end, &self.src.chars().nth(end).unwrap());
  // println!("start: {}:{:#?}", start, &self.src.chars().nth(start).unwrap());

  let (line, col) = line_col(src, start);

  // println!("offset: {offset}, start: {start}, end: {end}, line: {line}, col: {col}", offset=self.offset());
  // println!("on: {:#?}", &self.src.chars().nth(start).unwrap());

  let line_no_length = num_length(line as u32);
  let col_length = num_length(col as u32);

  format!(
    "{}:{}: {}\n{}^ here",
    line + 1, col + 1,
    &src[start..=end],
    "~".repeat(
      col + line_no_length + col_length
      + 3 // length of ":" and ": "
    )
  )
}

impl SourceReader<'_> {
  pub fn span_since(&self, start: usize) -> Span {
    Span { start, end: self.offset }
  }

  pub fn at(&self) -> String {
    at(&self.src, self.offset)
  }

  pub fn show_message(&self, message: Message) -> String {
    let (start_line, start_col) = line_col(&self.src, message.span.start);
    let (end_line, end_col) = line_col(&self.src, message.span.end);

    if start_line != end_line {
      todo!("multiline message");
    };

    let mut w: Vec<u8> = vec![];

    writeln!(&mut w, "{}: \x1b[1m{}\x1b[0m", message.level.to_string(), message.msg).unwrap();

    let pfx_len = num_length(start_line as u32 + 1);

    let Some((start, end)) = start_end(&self.src, message.span.start) else {
      return "<failed to write properly>".to_string();
    };

    // dbg!(start_line, start_col, end_line, end_col, start, end);

    writeln!(&mut w, "{} |",
      " ".repeat(pfx_len)
    ).unwrap();
    writeln!(&mut w, "{} | {}",
      start_line + 1,
      &self.src[start..=end]
    ).unwrap();
    writeln!(&mut w, "{} | {}{}",
      " ".repeat(pfx_len),
      " ".repeat(start_col),
      "^".repeat(end_col - start_col)
    ).unwrap();

    String::from_utf8(w).expect("failed to read from buffer")
  }
}
