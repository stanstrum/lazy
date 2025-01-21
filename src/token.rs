use snafu::{whatever, Whatever};

use super::*;

#[derive(Debug)]
pub(super) enum Token {}

#[derive(Debug)]
enum State {
  StartOfLine { indentation: String },
  Base,
}

impl State {
  fn new() -> Self {
    Self::StartOfLine {
      indentation: String::new(),
    }
  }
}

struct Span<'a> {
  file: &'a LazyFile,
}

pub(super) fn tokenize(file: &LazyFile) -> Result<impl Iterator<Item = Token>, Whatever> {
  let mut line = 1usize;
  let mut column = 1usize;

  // map the iter to really show him who's boss
  let mut iter = file.source_chars()?.map(move |(ch, index)| {
    let tuple = (ch, index, line, column);

    if ch == '\n' {
      line += 1;
      column = 1;
    } else {
      column += 1;
    };

    tuple
  });

  let mut state = State::new();

  Ok(
    iter.map(move |(ch, index, line, column)| match (&mut state, ch) {
      (State::StartOfLine { indentation }, ' ' | '\t') => todo!(),
      fallthrough => todo!("{fallthrough:#?}"),
    }),
  )
}
