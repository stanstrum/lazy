use snafu::prelude::*;

use super::ast::Span;

#[derive(Debug, Snafu)]
pub enum SourceReaderError {
  #[snafu(display("Invalid seek or rewind"))]
  InvalidSeekRewind,
}

pub struct SourceReader<'a> {
  src: &'a String,
  offset: usize
}

fn num_length<T: std::convert::Into<f64>>(n: T) -> usize {
  f64::from(
    (n.into().log10() + 1.0).floor().max(1.0)
  ) as usize
}

impl<'a> SourceReader<'a> {
  pub fn new(src: &'a String) -> Self {
    Self {
      src,
      offset: 0
    }
  }

  // pub fn src(&self) -> &'a String {
  //   &self.src
  // }

  pub fn offset(&self) -> usize {
    self.offset
  }

  pub fn len(&self) -> usize {
    self.src.len()
  }

  pub fn remaining(&self) -> usize {
    self.src.len() - self.offset
  }

  pub fn peek(&self, len: usize) -> Option<&'a str> {
    if self.remaining() >= len {
      Some(
        &self.src[self.offset..][..len]
      )
    } else {
      None
    }
  }

  pub fn seek(&mut self, len: usize) -> Result<(), SourceReaderError> {
    if self.remaining() >= len {
      self.offset += len;

      Ok(())
    } else {
      InvalidSeekRewindSnafu.fail()
    }
  }

  pub fn rewind(&mut self, len: usize) -> Result<(), SourceReaderError> {
    if self.offset >= len {
      self.offset -= len;

      Ok(())
    } else {
      InvalidSeekRewindSnafu.fail()
    }
  }

  pub fn read_ch(&mut self) -> Result<char, SourceReaderError> {
    if self.remaining() >= 1 {
      let ch = self.src.chars().nth(self.offset()).unwrap();

      self.offset += 1;

      Ok(ch)
    } else {
      InvalidSeekRewindSnafu.fail()
    }
  }

  pub fn read(&mut self, len: usize) -> Result<&'a str, SourceReaderError> {
    let Some(text) = self.peek(len) else {
      return InvalidSeekRewindSnafu.fail();
    };

    self.seek(len).unwrap();

    Ok(text)
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

  pub fn span_since(&self, start: usize) -> Span {
    Span { start, end: self.offset }
  }

  // pub fn wya_until(&self) -> String {
  //   todo!()
  // }
}
