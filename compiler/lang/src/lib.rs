pub mod span;
pub mod token;
pub mod intrinsic;

pub mod module;
pub mod function;

pub mod ty;
pub mod reference;

pub mod expr;

use std::fmt::Debug;

use string_pool::{PoolId, StringId};

#[derive(Debug, Clone, Copy)]
pub enum Token {
  Identifier(PoolId),
  Keyword(token::Keyword),
  Operator(token::Operator),
  Grouping(token::GroupingType),
  Whitespace,
  Indent(isize),
  Comment(StringId),
  Numeric(token::NumericValue),
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

pub trait Compiler: Debug + Sized
  where for<'a> Self::Store<'a>:
    reference::Store<Self::ModuleReference, Out = module::Module<Self>> +
    reference::Store<Self::TokensReference, Out = Self::Tokens> +
    //
    reference::Store<Self::StructReference, Out = Self::Struct> +
    reference::Store<Self::TypeReference, Out = ty::Type<Self>> +
    //
    reference::Store<Self::TypeReference, Out = ty::Type<Self>> +
    reference::Store<Self::TypePartReference, Out = ty::Type<Self>> +
    reference::Store<Self::OverwriteTypeReference, Out = ty::Type<Self>> +
    //
    reference::Store<Self::ExpressionReference, Out = expr::Expression<Self>> +
    reference::Store<Self::BlockReference, Out = expr::BlockExpression<Self>> +
{
  type Store<'a>;

  type ModuleReference: CompilerReference;

  type Tokens: Debug;
  type TokensReference: CompilerReference;

  type Struct: Debug;
  type StructReference: CompilerReference;

  type Function: Debug;
  type FunctionReference: CompilerReference;

  type TypeAlias: Debug;
  type TypeAliasReference: CompilerReference;

  type TypeReference: CompilerReference;
  type TypePartReference: CompilerReference;
  type OverwriteTypeReference: Debug + Clone;

  type VariableReference: CompilerReference;
  type BlockReference: CompilerReference;
  type ExpressionReference: CompilerReference;
}
