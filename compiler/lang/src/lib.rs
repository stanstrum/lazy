pub mod span;
pub mod special;
pub mod intrinsic;
pub mod ty;
pub mod module;
pub mod reference;

use std::fmt::Debug;

use string_pool::{PoolId, StringId};

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

pub trait CompilerReference: Debug + Clone + Copy + PartialEq + Eq {}
impl<T: Debug + Clone + Copy + PartialEq + Eq> CompilerReference for T {}

pub trait Compiler: Debug
  where for<'a> Self::Store<'a>:
    reference::Store<Self::ModuleReference, Out = Self::Module> +
    reference::Store<Self::TokensReference, Out = Self::Tokens> +
    reference::Store<Self::StructReference, Out = Self::Struct> +
    reference::Store<Self::OverwriteTypeReference, Out = Self::Type> +
{
  type Store<'a>;

  type Module: Debug;
  type ModuleReference: CompilerReference;

  type Tokens: Debug;
  type TokensReference: CompilerReference;

  type Struct: Debug;
  type StructReference: CompilerReference;

  type Type: Debug;
  type OverwriteTypeReference: Debug + Clone;

  // type Variable;
  // type VariableReference;
}
