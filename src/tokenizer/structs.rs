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
  BindAssign,
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

mod keywords {
  pub(super) const TYPE: &str = "type";
  pub(super) const IMPORT: &str = "import";
  pub(super) const EXPORT: &str = "export";
  pub(super) const FROM: &str = "from";
}

#[derive(Debug)]
pub(crate) enum Keyword {
  Type,
  Import,
  Export,
  From,
}

impl std::string::ToString for Keyword {
  fn to_string(&self) -> String {
    match self {
      Keyword::Type => keywords::TYPE,
      Keyword::Import => keywords::IMPORT,
      Keyword::Export => keywords::EXPORT,
      Keyword::From => keywords::FROM,
    }.to_string()
  }
}

impl TryFrom<String> for Keyword {
  type Error = ();

  fn try_from(value: String) -> Result<Self, Self::Error> {
    match value.as_str() {
      keywords::TYPE => Ok(Keyword::Type),
      keywords::IMPORT => Ok(Keyword::Import),
      keywords::EXPORT => Ok(Keyword::Export),
      keywords::FROM => Ok(Keyword::From),
      _ => Err(())
    }
  }
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum TokenEnum {
  Comment { ty: CommentType, content: String },
  Whitespace(String),
  Keyword(Keyword),
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

  // pub fn variant_mut<'a>(&'a mut self) -> &'a mut TokenEnum {
  //   &mut self.token
  // }
}
