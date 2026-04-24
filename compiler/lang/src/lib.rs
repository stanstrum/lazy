pub mod span;
pub mod token;
pub mod intrinsic;

pub mod module;
pub mod function;
pub mod import;

pub mod ty;
pub mod reference;

pub mod expr;

use std::fmt::Debug;

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
