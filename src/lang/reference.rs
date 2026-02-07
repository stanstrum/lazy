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

pub trait Store<'a, Item> where Self: 'a {
  type Out: 'a;

  fn rget(&self, key: Item) -> &Self::Out;
  fn rget_mut(&mut self, key: Item) -> &mut Self::Out;
}

impl<'a> Store<'a, ModuleReference> for Lazy<'a> {
  type Out = Module;

  fn rget(&self, ModuleReference(index): ModuleReference) -> &Self::Out {
    self.modules.get(index).unwrap()
  }

  fn rget_mut(&mut self, ModuleReference(index): ModuleReference) -> &mut Self::Out {
    self.modules.get_mut(index).unwrap()
  }
}

impl<'a> Store<'a, AliasReference> for Lazy<'a> {
  type Out = TypeAlias;

  fn rget(&self, AliasReference(module, index): AliasReference) -> &Self::Out {
    self.rget(module).aliases.get(index).unwrap()
  }

  fn rget_mut(&mut self, AliasReference(module, index): AliasReference) -> &mut Self::Out {
    self.rget_mut(module).aliases.get_mut(index).unwrap()
  }
}

impl<'a> Store<'a, FunctionReference> for Lazy<'a> {
  type Out = Function;

  fn rget(&self, FunctionReference(index): FunctionReference) -> &Self::Out {
    self.functions.get(index).unwrap()
  }

  fn rget_mut(&mut self, FunctionReference(index): FunctionReference) -> &mut Self::Out {
    self.functions.get_mut(index).unwrap()
  }
}

impl<'a> Store<'a, BlockReference> for Lazy<'a> {
  type Out = BlockExpression;

  fn rget(&self, BlockReference(function, id): BlockReference) -> &Self::Out {
    &self.rget(function)[id]
  }

  fn rget_mut(&mut self, BlockReference(function, id): BlockReference) -> &mut Self::Out {
    &mut self.rget_mut(function)[id]
  }
}

impl<'a> Store<'a, ExpressionReference> for Lazy<'a> {
  type Out = Expression;

  fn rget(&self, ExpressionReference(function, id): ExpressionReference) -> &Self::Out {
    &self.rget(function)[id]
  }

  fn rget_mut(&mut self, ExpressionReference(function, id): ExpressionReference) -> &mut Self::Out {
    &mut self.rget_mut(function)[id]
  }
}

impl<'a> Store<'a, TokensId> for Lazy<'a> {
  type Out = Vec<TokenSpan>;

  fn rget(&self, TokensId(index): TokensId) -> &Self::Out {
    self.tokens.get(index).unwrap()
  }

  fn rget_mut(&mut self, TokensId(index): TokensId) -> &mut Self::Out {
    self.tokens.get_mut(index).unwrap()
  }
}
