use crate::{Compiler, function::{BlockId, ExprId}};

pub trait Store<Item> {
  type Out;

  fn rget(&self, key: Item) -> &Self::Out;
  fn rget_mut(&mut self, key: Item) -> &mut Self::Out;
}

pub trait Reference<S: Store<Self>>: Sized {
  fn rget_from<'a>(&self, store: &'a S) -> &'a S::Out;
  fn rget_from_mut<'a>(&self, store: &'a mut S) -> &'a mut S::Out;
}

impl<R: Copy, S: Store<R>> Reference<S> for R {
  fn rget_from<'a>(&self, store: &'a S) -> &'a S::Out {
    store.rget(*self)
  }

  fn rget_from_mut<'a>(&self, store: &'a mut S) -> &'a mut S::Out {
    store.rget_mut(*self)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AliasReference<C: Compiler>(pub C::ModuleReference, pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructReference<C: Compiler>(pub C::ModuleReference, pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockReference<C: Compiler>(pub C::FunctionReference, pub BlockId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpressionReference<C: Compiler>(pub BlockReference<C>, pub ExprId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableReference<C: Compiler> {
  Block(BlockReference<C>, usize),
  Argument(C::FunctionReference, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypePartReference<C: Compiler>(pub C::ModuleReference, pub TypePartId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypePartId(pub(crate) usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeReference<C: Compiler> {
  Alias(AliasReference<C>),
  Part(TypePartReference<C>),
  Expression(ExpressionReference<C>),
  Block(BlockReference<C>),
  ReturnTypeOf(C::FunctionReference),
  Variable(VariableReference<C>),
  StructMember(StructReference<C>, usize),
}
