use std::io::Read;
use std::collections::VecDeque;

use crate::tokenize::{Tokenizer, token::TokenSpan};
use crate::lang::module::ModuleId;

use super::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mark(usize);

#[derive(Debug)]
pub(super) struct Rereader<'pool, const N: usize, T: Read> {
  pub id: ModuleId,
  queue: VecDeque<TokenSpan>,
  base: usize,
  index: usize,
  meta_reader: Tokenizer<'pool, N, T>,
}

impl<'a, 'pool, const N: usize, T: Read> Rereader<'pool, N, T> {
  pub(super) fn new(stream: Tokenizer<'pool, N, T>, id: ModuleId) -> Self {
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

  pub(super) fn mark(&self) -> Mark {
    Mark(self.index)
  }

  pub(super) fn take_mark(&mut self, Mark(index): Mark) {
    self.index = index;
  }

  pub(super) fn skip_to(&mut self, Mark(index): Mark) {
    for _ in 0..index {
      self.queue.pop_front();
    };

    self.base += index;
  }

  pub(super) fn peek(&mut self) -> Result<Option<TokenSpan>, Error> {
    let Some(index) = self.validate_index()? else {
      return Ok(None);
    };

    let tok = *self.queue.get(index).unwrap();
    let peek = Ok(Some(tok));
    dbg!(peek)
  }

  pub(super) fn seek(&mut self) {
    self.index += 1;
  }

  pub(super) fn ok_next(&mut self) -> Result<Option<TokenSpan>, Error> {
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
    let next = match self.validate_index() {
      Ok(Some(index)) => {
        let tok = *self.queue.get(index).unwrap();
        self.seek();

        Some(Ok(tok))
      },
      Ok(None) => None,
      Err(err) => Some(Err(err)),
    };

    dbg!(next)
  }
}
