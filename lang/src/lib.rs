pub mod span;
pub mod token;
pub mod intrinsic;

pub mod module;
pub mod function;
pub mod import;

pub mod ty;
pub mod reference;

pub mod expr;

use std::{fmt::Debug, hash::Hash};

pub trait CompilerReference: Debug + Clone + Copy + PartialEq + Eq {}
impl<T: Debug + Clone + Copy + PartialEq + Eq> CompilerReference for T {}

pub trait Compiler: Debug + Sized
  where for<'a> Self::Store<'a>:
    reference::Store<Self::ModuleReference, Out = module::Module<Self>> +
    reference::Store<Self::FunctionReference, Out = function::Function<Self>> +
    reference::Store<Self::TokensReference, Out = Self::Tokens> +
{
  type Store<'a>;

  type ModuleReference: CompilerReference + Hash;
  type FunctionReference: CompilerReference + Hash;

  type Tokens: Debug;
  type TokensReference: CompilerReference;

  type OverwriteTypeReference: Debug + Clone;
}
