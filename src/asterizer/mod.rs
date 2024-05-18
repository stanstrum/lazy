pub(crate) mod error;
pub(crate) mod ast;

pub(crate) use error::AsterizerError;

use std::fmt::Debug;

use ast::{
  GlobalNamespace,
  MakeAst
};

use crate::tokenizer::{
  Token,
  TokenEnum,
  Span
};

pub(crate) struct TokenStream {
  position: usize,
  marks: Vec<usize>,
  tokens: Vec<Token>,
  eof: bool,
}

impl TokenStream {
  pub fn new(tokens: Vec<Token>) -> Self {
    Self {
      position: 0,
      marks: vec![],
      tokens,
      eof: false
    }
  }

  pub fn span(&self) -> Span {
    self.peek()
      .map(|token| token.span.to_owned())
      .unwrap_or_else(|| {
        self.tokens.last()
          .map(|token| token.span.to_owned())
          .unwrap_or(Span {
            start: 0,
            end: 0
          })
      })
  }

  pub fn next(&mut self) -> Option<&Token> {
    if self.position >= self.tokens.len() {
      return None;
    };

    // We do this strangely because self.tokens.get later on creates
    // an immutable reference into the struct, disallowing a mut borrow
    // for self.seek afterwards
    let current_position = self.position;
    self.seek();

    let tok = self.tokens.get(current_position).unwrap();

    Some(tok)
  }

  pub fn seek(&mut self) {
    if self.position < self.tokens.len() - 1 {
      self.position += 1;
    } else {
      self.eof = true;
    };
  }

  pub fn next_variant(&mut self) -> Option<&TokenEnum> {
    self.next().map(Token::variant)
  }

  pub fn push_mark(&mut self) {
    self.marks.push(self.position);
  }

  pub fn pop_mark(&mut self) {
    if self.marks.is_empty() {
      return;
    };

    self.position = self.marks.pop().unwrap();
  }

  pub fn drop_mark(&mut self) {
    self.marks.pop().unwrap();
  }

  pub fn mark_len(&self) -> usize {
    self.marks.len()
  }

  pub fn peek(&self) -> Option<&Token> {
    if !self.eof {
      self.tokens.get(self.position)
    } else {
      None
    }
  }

  pub fn peek_variant(&self) -> Option<&TokenEnum> {
    self.peek().map(Token::variant)
  }

  pub fn skip_whitespace_and_comments(&mut self) {
    while let Some(TokenEnum::Comment { .. } | TokenEnum::Whitespace(..)) = self.peek_variant() {
      self.seek();
    };
  }

  pub fn remaining(&self) -> usize {
    self.tokens.len() - 1 - self.position
  }

  pub fn make<Ast: MakeAst + Debug>(&mut self) -> Result<Option<Ast>, AsterizerError> {
    let marks_len = self.mark_len();

    self.push_mark();

    // let type_name = Ast::type_name();

    // const PFX: &str = "lazy::asterizer::ast::";
    // let name = if type_name.starts_with(PFX) {
    //   type_name.strip_prefix(PFX).unwrap()
    // } else {
    //   type_name.as_str()
    // };

    // println!("make: {name}");

    let result = Ast::make(self);

    match &result {
      Ok(Some(_)) => {
        // println!("make: {name}: success");
        // dbg!(value);

        self.drop_mark();
      },
      Ok(None) => {
        // println!("make: {name}: none");

        self.pop_mark();
      },
      Err(_) => {
        // println!("make: {name}: error: {err}");

        // clean up marks in case of error
        loop {
          self.pop_mark();

          if marks_len == self.mark_len() {
            break;
          };
        };
      }
    };

    let new_marks_len = self.mark_len();

    assert!(new_marks_len == marks_len, "mark leak! {marks_len} -> {new_marks_len}");

    result
  }

  pub fn make_boxed<Ast: MakeAst + Debug>(&mut self) -> Result<Option<Box<Ast>>, AsterizerError> {
    Ok(self.make()?.map(Box::new))
  }
}

pub(crate) fn asterize(tokens: Vec<Token>) -> Result<GlobalNamespace, AsterizerError> {
  let mut stream = TokenStream::new(tokens);

  let Some(global) = stream.make()? else {
    panic!("no global made")
  };

  stream.skip_whitespace_and_comments();

  if stream.remaining() != 0 {
    println!("error: remaining tokens {}/{}", stream.position, stream.tokens.len());
    // dbg!(&stream.tokens[stream.position]);

    panic!("asterizer did not consume all tokens");
  };

  Ok(global)
}
