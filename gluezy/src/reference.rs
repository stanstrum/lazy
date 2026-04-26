use crate::{Lazy, LazyStructures};
use lang::reference::Store;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleReference(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionReference(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokensId(pub usize);

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
