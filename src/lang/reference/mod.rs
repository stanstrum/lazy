mod impls;

use crate::lang::Lazy;
use crate::lang::expr::{BlockExpression, Expression};
use crate::lang::function::{BlockId, ExprId, Function};
use crate::lang::module::{Module, TokensId, TypeAlias};
use crate::tokenize::token::TokenSpan;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FunctionReference(pub usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AliasReference(pub ModuleReference, pub usize);


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModuleReference(pub usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockReference(pub FunctionReference, pub BlockId);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExpressionReference(pub FunctionReference, pub ExprId);

#[derive(Debug)]
pub enum TypeReference {
  ReturnTypeOf(FunctionReference),
}

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

impl FunctionReference {
  pub fn body(&self, lazy: &Lazy) -> BlockReference {
    self.rget_from(lazy).body
  }

  pub fn get_body<'a>(&self, lazy: &'a Lazy) -> &'a BlockExpression {
    self.body(lazy).rget_from(lazy)
  }

  pub fn get_body_mut<'a>(&self, lazy: &'a mut Lazy) -> &'a mut BlockExpression {
    self.body(lazy).rget_from_mut(lazy)
  }

  pub fn last_expr(&self, lazy: &Lazy) -> Option<ExpressionReference> {
    let function = self.rget_from(lazy);
    let body = function.body.rget_from(lazy);

    body.returns_last.then(|| {
      let id = body.children.last().unwrap();
      ExpressionReference(*self, *id)
    })
  }
}
