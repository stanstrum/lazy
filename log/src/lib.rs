mod seek;
mod error;
mod print;

use std::io::{BufRead, BufReader, Write};
use std::fs::File;
use std::collections::HashMap;

use lazy_macros::{colorize, line_dbg};
use lang::token::Token;
use lang::span::{Position, Span};
use lang::Compiler;

pub use print::*;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
#[allow(unused)]
pub enum Level {
  Debug,
  Stub,
  Info,
  Warn,
  Error,
}

#[derive(Debug)]
pub struct PrintableMessage<C: Compiler> {
  pub level: Level,
  pub force: bool,
  pub description: String,
  pub contents: MessageContents<C>,
}

#[derive(Debug)]
pub struct WithinSource<C: Compiler> {
  pub range: Span<C>,
  pub sections: Vec<MessageSection<C>>,
}

#[derive(Debug)]
pub enum MessageContents<C: Compiler> {
  WithinSource(Vec<WithinSource<C>>),
  File(C::ModuleReference),
  None,
}

#[derive(Debug)]
pub struct MessageSection<C: Compiler> {
  pub text: String,
  pub span: Span<C>,
}

impl std::fmt::Display for Level {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      Level::Debug => concat!(              colorize!(36), "debug", colorize!(0)),
      Level::Stub =>  concat!(colorize!(7), colorize!(3),  "stub" , colorize!(0)),
      Level::Info =>  concat!(colorize!(7), colorize!(92), "info" , colorize!(0)),
      Level::Warn =>  concat!(colorize!(7), colorize!(93), "warn" , colorize!(0)),
      Level::Error => concat!(colorize!(7), colorize!(91), "error", colorize!(0)),
    })
  }
}

impl<C: Compiler> WithinSource<C> {
  pub fn new(sources: Vec<MessageSection<C>>) -> Vec<Self> {
    let mut map = HashMap::new();

    for section in sources {
      let list = map.entry(section.span.module).or_insert_with(Vec::new);
      list.push(section);
    };

    let mut results = Vec::with_capacity(map.len());
    for mut sources in map.into_values() {
      sources.sort_by_key(|section| section.span.start.position);

      assert!(!sources.is_empty());
      let start = sources.first().unwrap().span;
      let end = sources.last().unwrap().span;

      let range = Span::from_pair(start, end);

      results.push(WithinSource {
        range,
        sections: sources,
      });
    };

    results
  }
}

struct LineYielder {
  reader: BufReader<File>,
  line: usize,
  indentation: usize,
  finished: bool,
}

impl LineYielder {
  fn new(reader: BufReader<File>, line: usize) -> Self {
    Self {
      reader,
      line,
      indentation: 0,
      finished: false,
    }
  }

  fn rewind(&mut self, pos: Position) {
    seek::find_starting_newline(&mut self.reader, pos.position);
    self.line = pos.line;
  }
}

impl Iterator for LineYielder {
  type Item = String;

  fn next(&mut self) -> Option<Self::Item> {
    if self.finished {
      return None;
    };

    let mut buffer = String::new();
    let remaining = self.reader.read_line(&mut buffer)
      .expect("read line");

    if remaining == 0 {
      self.finished = true;
    };

    buffer.truncate(buffer.trim_end_matches(['\r', '\n']).len());

    self.indentation = buffer.chars()
      .position(|ch| !matches!(ch, ' ' | '\t'))
      .unwrap_or(0);

    self.line += 1;
    Some(buffer)
  }
}

struct Colorizer<'a, C: Compiler> {
  tokens: &'a [lang::token::TokenSpan<C>],
}

impl<'a, C: Compiler> Colorizer<'a, C> {
  fn colorize(token: &Token) -> &'static str {
    match token {
      Token::Identifier(_) => colorize!(93),
      Token::Keyword(_) => colorize!(91),
      Token::Operator(_) => colorize!(97),
      Token::Grouping(_) => colorize!(94),
      Token::Whitespace => colorize!(0),
      Token::Indent(_) => colorize!(36),
      Token::Comment(_) => colorize!(37),
      Token::Numeric(_) => colorize!(34),
      Token::String(..) => colorize!(31),
    }
  }

  fn find_token_by(&mut self, line: usize, column: usize) -> Option<&lang::token::TokenSpan<C>> {
    let offset = self.tokens.iter()
      .position(|(_, span)| span.start.line == line)?;

    self.tokens = &self.tokens[offset..];

    let tok_iter = self.tokens.iter()
      .take_while(|(_, span)| span.start.line == line);
    for token_span @ (_, span) in tok_iter {
      if column < span.start.column {
        continue;
      };

      if column >= span.end.column {
        continue;
      };

      return Some(token_span);
    };

    None
  }

  fn write(&mut self, mut out: impl Write, line: usize, text: &str) {
    let mut column = 1;
    while column <= text.len() {
      if let Some((token, span)) = self.find_token_by(line, column) {
        let color = Self::colorize(token);
        write!(out, "{color}").unwrap();

        let length = span.end.column - column;

        write!(out, "{}", &text[(column - 1)..][..length]).unwrap();
        write!(out, colorize!(0)).unwrap();
        column += length;

        continue;
      };

      write!(out, "{}", text.chars().nth(column - 1).unwrap()).unwrap();
      column += 1;
    };
  }
}
