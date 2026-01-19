use std::io::Read;
use std::collections::VecDeque;

use crate::line_dbg;
use crate::tokenize::token::Token;
use crate::tokenize::{Tokenizer, token::TokenSpan};
use crate::lang::module::ModuleId;

use super::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mark(usize);

#[derive(Debug)]
pub struct Rereader<'pool, const N: usize, T: Read> {
  pub id: ModuleId,
  pub queue: VecDeque<TokenSpan>,
  base: usize,
  index: usize,
  stream: Tokenizer<'pool, N, T>,
  indents: Vec<usize>,
}

impl<'pool, const N: usize, T: Read> Rereader<'pool, N, T> {
  pub fn new(stream: Tokenizer<'pool, N, T>, id: ModuleId) -> Self {
    Self {
      id,
      queue: VecDeque::new(),
      base: 0,
      index: 0,
      stream,
      indents: vec![],
    }
  }

  fn validate_index(&mut self) -> Result<Option<usize>, Error> {
    let index = self.index - self.base;

    while self.queue.len() < (index + 1) {
      let mut are_indents = false;

      loop {
        let Some(result) = self.stream.next() else {
          return Ok(None);
        };

        let tok = match result {
          Ok(tok) => tok,
          Err(err) => return Err(Error::Token(err)),
        };

        self.queue.push_back(tok);

        if !matches!(tok, (Token::Indent(_), _)) {
          break;
        };

        are_indents = true
      };

      if are_indents {
        let mut indents = self.queue.iter_mut()
          .rev()
          .skip(1)
          .take_while(|(tok, _)| matches!(tok, Token::Indent(_)))
          .map(|(tok, _)| {
            let Token::Indent(diff) = tok else { unreachable!() };
            diff
          }).collect::<Vec<_>>();

        if indents.len() < 2 {
          continue;
        };

        let sum = indents.iter().fold(0, |acc, x| acc + **x);

        let (last, rest) = indents.split_last_mut().unwrap();
        **last = sum;
        rest.into_iter().for_each(|rest| **rest = 0);
      };
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
    Ok(Some(tok))
  }

  pub(super) fn seek(&mut self) {
    self.index += 1;
    // eprintln!("{}: {}", line_dbg!("seek"), self.index)
  }

  pub(super) fn ok_next(&mut self) -> Result<Option<TokenSpan>, Error> {
    match self.next() {
      Some(Ok(token)) => Ok(Some(token)),
      Some(Err(err)) => Err(err),
      None => Ok(None),
    }
  }

  pub(super) fn examine_tokens(self) -> VecDeque<TokenSpan> {
    self.queue
  }
}

impl<'pool, const N: usize, T: Read> Iterator for Rereader<'pool, N, T> {
  type Item = Result<TokenSpan, Error>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.validate_index() {
      Ok(Some(index)) => {
        let tok = *self.queue.get(index).unwrap();
        self.seek();

        Some(Ok(tok))
      },
      Ok(None) => None,
      Err(err) => Some(Err(err)),
    }
  }
}
