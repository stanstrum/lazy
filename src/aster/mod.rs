mod make;

use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;

use crate::bufreader::BufferedUtf8MetadataReader;
use crate::string_pool::StringPool;

use crate::tokenize::{self, Tokenizer};
use crate::tokenize::token::{Span, TokenSpan};
use crate::lang::Lazy;
use crate::lang::module::ModuleId;

#[derive(Debug)]
struct Rereader<'pool, const N: usize, T: Read> {
  id: ModuleId,
  queue: VecDeque<TokenSpan>,
  base: usize,
  index: usize,
  meta_reader: Tokenizer<'pool, N, T>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mark(usize);

impl<'a, 'pool, const N: usize, T: Read> Rereader<'pool, N, T> {
  fn new(stream: Tokenizer<'pool, N, T>, id: ModuleId) -> Self {
    Self {
      id,
      queue: VecDeque::new(),
      base: 0,
      index: 0,
      meta_reader: stream,
    }
  }

  fn validate_index(&mut self) -> Result<Option<usize>, Error> {
    let index = self.index - self.base;
    while self.queue.len() < (index + 1) {
      let Some(result) = self.meta_reader.next() else {
        return Ok(None);
      };

      let tok = match result {
        Ok(tok) => tok,
        Err(err) => return Err(Error::Token(err)),
      };

      self.queue.push_back(tok);
    };

    Ok(Some(index))
  }

  fn mark(&self) -> Mark {
    Mark(self.index)
  }

  fn take_mark(&mut self, Mark(index): Mark) {
    self.index = index;
  }

  fn skip_to(&mut self, Mark(index): Mark) {
    for _ in 0..index {
      self.queue.pop_front();
    };

    self.base += index;
  }

  fn peek(&mut self) -> Result<Option<TokenSpan>, Error> {
    let Some(index) = self.validate_index()? else {
      return Ok(None);
    };

    let tok = *self.queue.get(index).unwrap();
    dbg!(/* peek */ Ok(Some(tok)))
  }

  fn seek(&mut self) {
    self.index += 1;
  }

  fn ok_next(&mut self) -> Result<Option<TokenSpan>, Error> {
    match self.next() {
      Some(Ok(token)) => Ok(Some(token)),
      Some(Err(err)) => Err(err),
      None => Ok(None),
    }
  }
}

impl<'pool, const N: usize, T: Read> Iterator for Rereader<'pool, N, T> {
  type Item = Result<TokenSpan, Error>;

  fn next(&mut self) -> Option<Self::Item> {
    let value = match self.validate_index() {
      Ok(Some(index)) => {
        let tok = *self.queue.get(index).unwrap();
        self.seek();

        Some(Ok(tok))
      },
      Ok(None) => None,
      Err(err) => Some(Err(err)),
    };

    dbg!(/* next */ value)
  }
}

#[derive(Debug)]
pub enum Error {
  Token(tokenize::Error),
  Expected {
    what: &'static str,
    at: Span,
  },
}

pub fn asterize<'pool>(lazy: &mut Lazy<'pool>, pool: &'pool StringPool, id: ModuleId) -> Result<(), Error> {
  let path = lazy.get_path(id);
  let file = File::open(path).expect("failed to open path");
  let meta_reader = BufferedUtf8MetadataReader::<64, _>::new(file);
  let tokenizer = Tokenizer::new(pool, id, meta_reader);
  let mut rereader = Rereader::new(tokenizer, id);

  make::make(lazy, &mut rereader)?;

  Ok(())
}
