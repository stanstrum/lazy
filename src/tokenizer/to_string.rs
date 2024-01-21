/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::string::ToString;

use super::{
  Token,
  TokenEnum,
  CommentType,
  Operator,
  Punctuation,
  Grouping,
  GroupingType
};

impl ToString for Token {
  fn to_string(&self) -> String {
    self.token.to_string()
  }
}

impl ToString for TokenEnum {
  fn to_string(&self) -> String {
    match self {
      Self::Comment { ty: CommentType::Multiline, content } => {
        format!("/*{content}*/")
      },
      Self::Comment { ty: CommentType::Line, content } => {
        format!("//{content}\n")
      },
      Self::Whitespace(content)
      | Self::Identifier(content) => {
        content.to_owned()
      },
      Self::Operator(Operator::RightArrow) => "->".to_owned(),
      Self::Punctuation(Punctuation::Semicolon) => ";".to_owned(),
      Self::Punctuation(Punctuation::Colon) => ":".to_owned(),
      Self::Punctuation(Punctuation::Comma) => ",".to_owned(),
      Self::Grouping(Grouping::Open(GroupingType::CurlyBrace)) => "{".to_owned(),
      Self::Grouping(Grouping::Close(GroupingType::CurlyBrace)) => "}".to_owned(),
      _ => todo!("{self:#?}")
    }
  }
}
