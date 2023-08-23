/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::{SourceReader, Span};
use std::io::Write;

use crate::colors::*;

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
        format!("{BOLD}{MAGENTA}debug{CLEAR}")
      },
      Level::Note => {
        format!("{BOLD}{CYAN}note{CLEAR}")
      },
      Level::Warning => {
        format!("{BOLD}{YELLOW}warning{CLEAR}")
      },
      Level::Error => {
        format!("{BOLD}{RED}error{CLEAR}")
      },
    }
  }
}

pub struct Message {
  pub level: Level,
  pub msg: String,
  pub sub: String,
  pub span: Span,
}

pub fn start_end(src: &String, offset: usize) -> (usize, usize) {
  let mut start = offset;
  let mut end = offset;

  loop {
    if start == 0 {
      break;
    };

    if src.chars().nth(start).unwrap() == '\n' {
      break;
    };

    start -= 1;
  };

  loop {
    if end == 0 {
      break;
    };

    if src.chars().nth(end).unwrap() == '\n' {
      break;
    };

    end += 1;
  };

  (start, end)
}

pub fn line_col(src: &String, offset: usize) -> (usize, usize) {
  let (start, _) = start_end(src, offset);

  let mut line: usize = 0;
  for ch in src[..=start].chars() {
    if ch == '\n' {
      line += 1;
    };
  };

  let col = src[start..=offset].chars().filter(|ch| *ch != '\n').count();

  (line, if col > 0 { col - 1 } else { 0 })
}

fn at(src: &String, offset: usize) -> String {
  // seek to beginning of line
  let (start, end) = start_end(src, offset);

  println!("end: {}:{:#?}", end, src.chars().nth(end).unwrap());
  println!("start: {}:{:#?}", start, src.chars().nth(start).unwrap());

  let (line, col) = line_col(src, start);

  println!("offset: {offset}, start: {start}, end: {end}, line: {line}, col: {col}");
  println!("on: {:#?}", src.chars().nth(start).unwrap());

  let line_no_length = num_length(line as u32 + 1);
  let col_length = num_length(col as u32);

  format!(
    "{}:{}: {}\n{}^ here",
    line + 1, col,
    get_code(src, offset),
    "~".repeat(
      col + line_no_length + col_length
      + 3 // length of ":" and ": "
    )
  )
}

fn get_code<'a>(src: &'a String, offset: usize) -> &'a str {
  let (mut start, mut end) = start_end(src, offset);

  if end >= src.len() {
    panic!("end offset is out of range");
  };

  while src.chars().nth(start).unwrap() == '\n' {
    start += 1;
  };

  while src.chars().nth(end).unwrap() == '\n' {
    end -= 1;
  };

  if start <= end {
    &src[start..=end]
  } else {
    ""
  }
}

pub fn format_message(src: &String, message: Message) -> String {
  let (start_line, start_col) = line_col(src, message.span.start);
  let (end_line, end_col) = line_col(src, message.span.end);

  // dbg!(start_line, start_col, end_line, end_col);

  let mut w: Vec<u8> = vec![];

  if start_line != end_line {
    todo!("multiline message")
  };

  writeln!(&mut w, "{}: {BOLD}{}{CLEAR}", message.level.to_string(), message.msg).unwrap();

  let pfx_len = num_length(start_line as u32 + 1);

  writeln!(&mut w, "{} |",
    " ".repeat(pfx_len)
  ).unwrap();
  writeln!(&mut w, "{} | {}",
    start_line + 1,
    get_code(src, message.span.start)
  ).unwrap();
  writeln!(&mut w, "{} | {}{} {}",
    " ".repeat(pfx_len),
    " ".repeat(start_col),
    "─".repeat((end_col - start_col).max(1)),
    message.sub
  ).unwrap();

  String::from_utf8(w).expect("failed to read from buffer")
}

impl SourceReader<'_> {
  pub fn span_since(&self, start: usize) -> Span {
    Span { start, end: self.offset }
  }

  pub fn at(&self) -> String {
    at(&self.src, self.offset)
  }
}
