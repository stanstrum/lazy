use crate::{Compiler, expr::{BlockExpression, Expression}, function::{BlockId, ExprId}, ty::{OverwriteTypeReference, Type, TypeOf, TypePair}};

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

pub trait FunctionGetBody<C: Compiler> {
  fn body(&self) -> BlockReference<C>;
  // fn get_body<'a>(&self, lazy: &'a Lazy) -> &'a BlockExpression;

  fn get_body_mut<'store, 'pool>(&self, store: &'store mut C::Store<'pool>) -> &'store mut BlockExpression<C>;

  // fn last_expr(&self, lazy: &Lazy) -> Option<ExpressionReference>;
}

impl<C: Compiler> FunctionGetBody<C> for C::FunctionReference {
  fn body(&self) -> BlockReference<C> {
    BlockReference::<C>(*self, BlockId::body_id())
  }

  // fn get_body<'a>(&self, lazy: &'a Lazy) -> &'a BlockExpression {
  //   self.body(lazy).rget_from(lazy)
  // }

  fn get_body_mut<'store, 'pool>(&self, store: &'store mut C::Store<'pool>) -> &'store mut BlockExpression<C> {
    self.body().rget_from_mut(store)
  }

  // fn last_expr(&self, lazy: &Lazy) -> Option<ExpressionReference> {
  //   let function = self.rget_from(lazy);
  //   let body = function.body.rget_from(lazy);

  //   body.returns_last.then(|| {
  //     let id = body.children.last().unwrap();
  //     ExpressionReference(*self, *id)
  //   })
  // }
}
