use ::token::Position;

pub mod token;
mod numeric;
mod state;

use std::collections::VecDeque;
use std::io::Read;

use crate::lang::reference::ModuleReference;
use crate::aster::bufreader::{BufferedUtf8MetadataReader};
use string_pool::StringPool;

use state::State;
use token::{Span, Token, TokenSpan};

#[derive(Debug)]
pub struct Tokenizer<'pool, const N: usize, T: Read> {
  pool: &'pool StringPool,
  module: ModuleReference,
  name: String,
  meta_reader: BufferedUtf8MetadataReader<N, T>,
  state: State,
  next: Option<char>,
  toks: VecDeque<TokenSpan>,
  indentation: isize,
  ended: bool,
  last_span: Span,
  override_indentation: Option<usize>,
}

#[derive(Debug)]
pub enum Error {
  IO {
    module: ModuleReference,
    name: String,
  },
  InvalidNumeric {
    span: Span,
  },
}

impl<'pool, const N: usize, T: Read> Tokenizer<'pool, N, T> {
  pub fn new(pool: &'pool StringPool, module: ModuleReference, name: String, meta_reader: BufferedUtf8MetadataReader<N, T>) -> Self {
    let start_and_end = Position::new();

    Self {
      pool,
      module,
      name,
      meta_reader,
      state: State::Base,
      next: None,
      toks: VecDeque::new(),
      indentation: 0,
      ended: false,
      last_span: Span {
        module,
        start: start_and_end,
        end: start_and_end,
      },
      override_indentation: None,
    }
  }

  fn end_indent(&mut self) -> Option<Result<TokenSpan, Error>> {
    if self.ended {
      None
    } else {
      self.ended = true;
      let mut span = self.last_span;
      span.start.indentation = 0;
      span.end.indentation = 0;
      Some(Ok((Token::Indent(-self.indentation), span)))
    }
  }

  pub(super) fn take_ch(&mut self) -> Result<Option<char>, Error> {
    if let Some(next) = self.next.take() {
      return Ok(Some(next));
    };

    match self.meta_reader.next() {
      Some(Ok(ch)) => Ok(Some(ch)),
      Some(Err(_)) => Err(Error::IO {
        name: self.name.to_owned(),
        module: self.module,
      }),
      None => Ok(None),
    }
  }

  fn save(&mut self, ch: char) {
    assert!(self.next.is_none());

    self.next = Some(ch);
  }

  fn retry(&mut self, ch: char, state: State) {
    self.save(ch);
    self.state = state;
  }

  fn push_here(&mut self, tok: Token, start: Position) {
    let span = Span {
      start,
      end: self.pos(),
      module: self.module,
    };
    self.toks.push_back((tok, span));
  }

  fn next_pos(&mut self) -> Result<Position, Error> {
    if let Some(ch) = self.take_ch()? {
      self.save(ch);
    };

    Ok(self.pos())
  }

  fn pos(&self) -> Position {
    (&self.meta_reader.meta).into()
  }
}

impl<'a, const N: usize, T: Read> Iterator for Tokenizer<'a, N, T> {
  type Item = Result<TokenSpan, Error>;

  fn next(&mut self) -> Option<Self::Item> {
    let result = self.do_state();

    if let Some(Ok((_, span))) = result {
      self.last_span = span;
    };

    result.or_else(|| self.end_indent())
  }
}
