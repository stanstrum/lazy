/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Span {
  pub start: usize,
  pub end: usize
}

#[derive(Debug)]
pub(crate) enum Operator {
  RightArrow,
}

#[derive(Debug)]
pub(crate) enum Punctuation {
  Colon,
  Comma,
  Semicolon
}

#[derive(Debug)]
pub(crate) enum CommentType {
  Line,
  Multiline
}

#[derive(Debug)]
pub(crate) enum TokenEnum {
  Comment { ty: CommentType, content: String },
  Whitespace(String),
  Keyword,
  Identifier(String),
  Operator(Operator),
  Punctuation(Punctuation),
  Grouping(Grouping),
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Token {
  pub token: TokenEnum,
  pub span: Span
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum GroupingType {
  Parenthesis,
  Bracket,
  CurlyBrace
}

#[derive(Debug)]
pub(crate) enum Grouping {
  Open(GroupingType),
  Close(GroupingType)
}

impl Token {
  pub fn variant<'a>(&'a self) -> &'a TokenEnum {
    &self.token
  }

  pub fn variant_mut<'a>(&'a mut self) -> &'a mut TokenEnum {
    &mut self.token
  }
}
