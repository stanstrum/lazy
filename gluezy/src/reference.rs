use crate::{Lazy, LazyStructures, prelude::module::TokensId};
use ::lang::expr::BlockExpression;
use lang::{Compiler, function::{BlockId, ExprId}, reference::{BlockReference, Reference, Store}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleReference(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionReference(pub usize);

pub type TypePartReference = ::lang::reference::TypePartReference<LazyStructures>;
pub type TypeReference = ::lang::reference::TypeReference<LazyStructures>;

pub type VariableReference = ::lang::reference::VariableReference<LazyStructures>;

impl<'pool> Store<ModuleReference> for Lazy<'pool> {
  type Out = lang::module::Module<LazyStructures>;

  fn rget(&self, ModuleReference(index): ModuleReference) -> &Self::Out {
    self.modules.get(index).unwrap()
  }

  fn rget_mut(&mut self, ModuleReference(index): ModuleReference) -> &mut Self::Out {
    self.modules.get_mut(index).unwrap()
  }
}

impl<'pool> Store<FunctionReference> for Lazy<'pool> {
  type Out = lang::function::Function<LazyStructures>;

  fn rget(&self, FunctionReference(index): FunctionReference) -> &Self::Out {
    self.functions.get(index).unwrap()
  }

  fn rget_mut(&mut self, FunctionReference(index): FunctionReference) -> &mut Self::Out {
    self.functions.get_mut(index).unwrap()
  }
}

impl<'pool> Store<TokensId> for Lazy<'pool> {
  type Out = Vec<lang::token::TokenSpan<LazyStructures>>;

  fn rget(&self, TokensId(index): TokensId) -> &Self::Out {
    self.tokens.get(index).unwrap()
  }

  fn rget_mut(&mut self, TokensId(index): TokensId) -> &mut Self::Out {
    self.tokens.get_mut(index).unwrap()
  }
}

impl FunctionReference {
  pub fn body(&self) -> BlockReference<LazyStructures> {
    BlockReference(*self, BlockId::body_id())
  }

  // pub fn get_body<'a>(&self, lazy: &'a Lazy) -> &'a BlockExpression {
  //   self.body(lazy).rget_from(lazy)
  // }

  pub fn get_body_mut<'a>(&self, lazy: &'a mut Lazy) -> &'a mut BlockExpression<LazyStructures> {
    self.body().rget_from_mut(lazy)
  }

  // pub fn last_expr(&self, lazy: &Lazy) -> Option<ExpressionReference> {
  //   let function = self.rget_from(lazy);
  //   let body = function.body.rget_from(lazy);

  //   body.returns_last.then(|| {
  //     let id = body.children.last().unwrap();
  //     ExpressionReference(*self, *id)
  //   })
  // }
}
