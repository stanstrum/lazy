use std::string::ToString;

use crate::tokenizer::{
  Token,
  TokenEnum,
  Punctuation,
  Operator,
  Literal,
  CommentType,
};

impl ToString for Token {
  fn to_string(&self) -> String {
    self.token.to_string()
  }
}

fn escape_string(original: &str) -> String {
  let mut escaped = String::new();

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
      Self::Operator(Operator::Modulo) => "%".to_owned(),
      Self::Operator(Operator::Divide) => "+".to_owned(),
      Self::Operator(Operator::Equality) => "==".to_owned(),
      Self::Operator(Operator::Equals) => "=".to_owned(),
      Self::Operator(Operator::Range) => "..".to_owned(),
      Self::Operator(Operator::Dot) => ".".to_owned(),
      Self::Operator(Operator::Separator) => "::".to_owned(),
      Self::Operator(Operator::LogicalShiftRightAssign) => ">>>=".to_owned(),
      Self::Operator(Operator::LogicalShiftRight) => ">>>".to_owned(),
      Self::Operator(Operator::ShiftRightAssign) => ">>=".to_owned(),
      Self::Operator(Operator::ShiftRight) => ">>".to_owned(),
      Self::Operator(Operator::GreaterThanEqual) => ">=".to_owned(),
      Self::Operator(Operator::GreaterThan) => ">".to_owned(),
      Self::Operator(Operator::ShiftLeftAssign) => "<<=".to_owned(),
      Self::Operator(Operator::ShiftLeft) => "<<".to_owned(),
      Self::Operator(Operator::LessThanEqual) => "<=".to_owned(),
      Self::Operator(Operator::LessThan) => "<".to_owned(),
      Self::Punctuation(Punctuation::Semicolon) => ";".to_owned(),
      Self::Punctuation(Punctuation::Colon) => ":".to_owned(),
      Self::Punctuation(Punctuation::Comma) => ",".to_owned(),
      Self::Punctuation(Punctuation::VariadicEllipsis) => "...".to_owned(),
      Self::Literal(Literal::Integer(value)) => value.to_string(),
      Self::Literal(Literal::FloatingPoint(value)) => value.to_string(),
      Self::Literal(Literal::UnicodeString(content)) => format!("\"{}\"", escape_string(content)),
      Self::Literal(Literal::CString(content)) => format!("c\"{}\"", escape_string(content)),
      Self::Literal(Literal::ByteString(content)) => format!("b\"{}\"", escape_string(content)),
      Self::Literal(Literal::UnicodeChar(content)) => format!("'{}'", escape_string(&String::from(*content))),
      Self::Literal(Literal::ByteChar(content)) => format!("b'{}'", escape_string(&String::from(*content as char))),
      // --
      Self::Grouping(grouping) => grouping.to_string(),
      Self::Keyword(keyword) => keyword.to_string(),
      Self::Invalid(rest_of_line) => rest_of_line.to_string(),
    }
  }
}
