use string_pool::{PoolId, StringId};

pub mod span;
pub mod special;

#[derive(Debug, Clone, Copy)]
pub enum Token {
  Identifier(PoolId),
  Keyword(special::Keyword),
  Operator(special::Operator),
  Grouping(special::GroupingType),
  Whitespace,
  Indent(isize),
  Comment(StringId),
  Numeric(special::NumericValue),
  String(StringKind, StringId)
}

#[derive(Debug, Clone, Copy)]
pub enum StringKind {
  Wide,
  Byte,
  C,
}

#[derive(Debug, Clone, Copy)]
pub enum CharKind {
  Wide,
  Byte,
}
