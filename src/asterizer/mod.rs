/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::tokenizer::{Token, TokenEnum};

pub(crate) mod error;
pub(crate) use error::AsterizerError;

pub(crate) mod ast;

use ast::{GlobalNamespace, MakeAst};

#[derive(Debug)]
enum State {
  TopLevel,
  Function,
}

pub(crate) struct TokenStream {
  position: usize,
  marks: Vec<usize>,
  tokens: Vec<Token>
}

impl TokenStream {
  pub fn new(tokens: Vec<Token>) -> Self {
    Self {
      position: 0,
      marks: vec![],
      tokens
    }
  }

  pub fn next<'a>(&'a mut self) -> Option<&'a Token> {
    if self.position >= self.tokens.len() {
      return None;
    };

    let tok = self.tokens.get(self.position).unwrap();

    self.position += 1;

    Some(tok)
  }

  pub fn next_variant<'a>(&'a mut self) -> Option<&'a TokenEnum> {
    self.next().map(Token::variant)
  }

  pub fn push_mark(&mut self) {
    self.marks.push(self.position);
  }

  pub fn pop_mark(&mut self) {
    self.position = self.marks.pop().unwrap();
  }

  pub fn drop_mark(&mut self) {
    self.marks.pop().unwrap();
  }

  pub fn peek<'a>(&'a self) -> Option<&'a Token> {
    self.tokens.get(self.position)
  }

  pub fn peek_variant<'a>(&'a self) -> Option<&'a TokenEnum> {
    self.peek().map(Token::variant)
  }

  pub fn skip_whitespace_and_comments(&mut self) {
    loop {
      match self.peek_variant() {
        Some(TokenEnum::Comment { .. } | TokenEnum::Whitespace(..)) => {
          self.position += 1;
        },
        _ => break
      };
    }
  }
}

pub(crate) fn asterize(tokens: Vec<Token>) -> Result<GlobalNamespace, AsterizerError> {
  let mut stream = TokenStream::new(tokens);

  let Some(global) = GlobalNamespace::make(&mut stream)? else {
    panic!("no global made")
  };

  Ok(global)
}
