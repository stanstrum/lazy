use string_pool::{PoolId, StringId};

pub mod span;
pub mod special;
pub mod intrinsic;
pub mod ty;
pub mod module;
pub mod reference;

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

impl From<StringKind> for intrinsic::Intrinsic {
  fn from(value: StringKind) -> Self {
    match value {
      StringKind::Wide => Self::U32,
      StringKind::Byte | StringKind::C => Self::U8,
    }
  }
}

trait Compiler {
  type Module;
}
