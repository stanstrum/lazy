use std::string::ToString;

use crate::tokenizer::{
  Token,
  TokenEnum,
  CommentType,
  Operator,
  Punctuation,
  Literal
};

impl ToString for Token {
  fn to_string(&self) -> String {
    self.token.to_string()
  }
}

fn escape_string(original: &str, prefix: Option<char>) -> String {
  let mut escaped = String::new();

  if let Some(prefix) = prefix {
    escaped.push(prefix);
  };

  escaped.push('"');

  for ch in original.chars() {
    match ch {
      '\n' => escaped += "\\n",
      '\t' => escaped += "\\t",
      '\r' => escaped += "\\r",
      '\\' => escaped += "\\\\",
      '\'' => escaped += "\\'",
      '\0' => escaped += "\\0",
      // TODO: add more escape codes
      _ => escaped.push(ch)
    };
  };

  escaped.push('"');

  escaped
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
      // TODO: make ToString/TryFrom<String> for these and dry this up
      Self::Operator(Operator::RightArrow) => "->".to_owned(),
      Self::Operator(Operator::BindAssign) => ":=".to_owned(),
      Self::Operator(Operator::SingleAnd) => "&".to_owned(),
      Self::Operator(Operator::Increment) => "++".to_owned(),
      Self::Operator(Operator::Add) => "+".to_owned(),
      Self::Operator(Operator::Decrement) => "--".to_owned(),
      Self::Operator(Operator::Subtract) => "-".to_owned(),
      Self::Operator(Operator::Exponent) => "**".to_owned(),
      Self::Operator(Operator::Multiply) => "*".to_owned(),
      Self::Operator(Operator::Divide) => "+".to_owned(),
      Self::Operator(Operator::Equality) => "==".to_owned(),
      Self::Operator(Operator::Equals) => "=".to_owned(),
      Self::Operator(Operator::Range) => "..".to_owned(),
      Self::Operator(Operator::Dot) => ".".to_owned(),
      Self::Operator(Operator::Separator) => "::".to_owned(),
      Self::Punctuation(Punctuation::Semicolon) => ";".to_owned(),
      Self::Punctuation(Punctuation::Colon) => ":".to_owned(),
      Self::Punctuation(Punctuation::Comma) => ",".to_owned(),
      Self::Literal(Literal::Integer(value)) => value.to_string(),
      Self::Literal(Literal::FloatingPoint(value)) => value.to_string(),
      Self::Literal(Literal::UnicodeString(content)) => escape_string(content, None),
      Self::Literal(Literal::CString(content)) => escape_string(content, Some('c')),
      Self::Literal(Literal::ByteString(content)) => escape_string(content, Some('b')),
      // --
      Self::Grouping(grouping) => grouping.to_string(),
      Self::Keyword(keyword) => keyword.to_string(),
    }
  }
}
