pub mod token;
mod numeric;
mod state;

use std::collections::VecDeque;
use std::io::Read;

use crate::lang::module::ModuleId;
use crate::aster::bufreader::{BufferedUtf8MetadataReader};
use crate::string_pool::StringPool;

use state::State;
use token::{Position, Span, Token, TokenSpan};

#[derive(Debug)]
pub struct Tokenizer<'pool, const N: usize, T: Read> {
  pool: &'pool StringPool,
  id: ModuleId,
  meta_reader: BufferedUtf8MetadataReader<N, T>,
  state: State,
  next: Option<char>,
  toks: VecDeque<TokenSpan>,
  indentation: isize,
  ended: bool,
  last_span: Span,
}

#[derive(Debug)]
pub enum Error {
  IO,
  InvalidNumeric,
}

impl<'pool, const N: usize, T: Read> Tokenizer<'pool, N, T> {
  pub fn new(pool: &'pool StringPool, id: ModuleId, meta_reader: BufferedUtf8MetadataReader<N, T>) -> Self {
    let start_and_end = Position::new();

    Self {
      pool,
      id,
      meta_reader,
      state: State::Base,
      next: None,
      toks: VecDeque::new(),
      indentation: 0,
      ended: false,
      last_span: Span {
        module: id,
        start: start_and_end,
        end: start_and_end,
      },
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

  fn take(&mut self) -> Result<Option<char>, Error> {
    if let Some(next) = self.next.take() {
      return Ok(Some(next));
    };

    match self.meta_reader.next() {
      Some(Ok(ch)) => Ok(Some(ch)),
      Some(Err(_)) => Err(Error::IO),
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
      module: self.id,
    };
    self.toks.push_back((tok, span));
  }

  fn pos(&self) -> Position {
    Position::new_from_meta(&self.meta_reader.meta)
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
