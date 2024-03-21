use std::string::ToString;

use crate::tokenizer::{
  Token,
  TokenEnum,
  CommentType,
  Operator,
  Punctuation,
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
      Self::Operator(Operator::BindAssign) => ":=".to_owned(),
      Self::Punctuation(Punctuation::Semicolon) => ";".to_owned(),
      Self::Punctuation(Punctuation::Colon) => ":".to_owned(),
      Self::Punctuation(Punctuation::Comma) => ",".to_owned(),
      Self::Grouping(grouping) => grouping.to_string(),
      Self::Keyword(keyword) => keyword.to_string(),
    }
  }
}
