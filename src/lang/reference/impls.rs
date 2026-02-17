use crate::lang::{expr::Variable, ty::Type};

use super::*;

impl<'a> Store<ModuleReference> for Lazy<'a> {
  type Out = Module;

  fn rget(&self, ModuleReference(index): ModuleReference) -> &Self::Out {
    self.modules.get(index).unwrap()
  }

  fn rget_mut(&mut self, ModuleReference(index): ModuleReference) -> &mut Self::Out {
    self.modules.get_mut(index).unwrap()
  }
}

impl<'a> Store<AliasReference> for Lazy<'a> {
  type Out = TypeAlias;

  fn rget(&self, AliasReference(module, index): AliasReference) -> &Self::Out {
    self.rget(module).aliases.get(index).unwrap()
  }

  fn rget_mut(&mut self, AliasReference(module, index): AliasReference) -> &mut Self::Out {
    self.rget_mut(module).aliases.get_mut(index).unwrap()
  }
}

impl<'a> Store<FunctionReference> for Lazy<'a> {
  type Out = Function;

  fn rget(&self, FunctionReference(index): FunctionReference) -> &Self::Out {
    self.functions.get(index).unwrap()
  }

  fn rget_mut(&mut self, FunctionReference(index): FunctionReference) -> &mut Self::Out {
    self.functions.get_mut(index).unwrap()
  }
}

impl<'a> Store<BlockReference> for Lazy<'a> {
  type Out = BlockExpression;

  fn rget(&self, BlockReference(function, id): BlockReference) -> &Self::Out {
    &self.rget(function)[id]
  }

  fn rget_mut(&mut self, BlockReference(function, id): BlockReference) -> &mut Self::Out {
    &mut self.rget_mut(function)[id]
  }
}

impl<'a> Store<ExpressionReference> for Lazy<'a> {
  type Out = Expression;

  fn rget(&self, ExpressionReference(function, id): ExpressionReference) -> &Self::Out {
    &self.rget(function)[id]
  }

  fn rget_mut(&mut self, ExpressionReference(function, id): ExpressionReference) -> &mut Self::Out {
    &mut self.rget_mut(function)[id]
  }
}

impl<'a> Store<TokensId> for Lazy<'a> {
  type Out = Vec<TokenSpan>;

  fn rget(&self, TokensId(index): TokensId) -> &Self::Out {
    self.tokens.get(index).unwrap()
  }

  fn rget_mut(&mut self, TokensId(index): TokensId) -> &mut Self::Out {
    self.tokens.get_mut(index).unwrap()
  }
}

impl<'a> Store<TypePartReference> for Lazy<'a> {
  type Out = Type;

  fn rget(&self, TypePartReference(module, TypePartId(index)): TypePartReference) -> &Self::Out {
    self.rget(module).type_parts.get(index).unwrap()
  }

  fn rget_mut(&mut self, TypePartReference(module, TypePartId(index)): TypePartReference) -> &mut Self::Out {
    self.rget_mut(module).type_parts.get_mut(index).unwrap()
  }
}

impl<'a> Store<VariableReference> for Lazy<'a> {
  type Out = Variable;

  fn rget(&self, key: VariableReference) -> &Self::Out {
    match key {
      VariableReference::Block(block, index) => {
        self.rget(block).variables.get(index).unwrap()
      },
      VariableReference::Argument(function, index) => todo!(),
    }
  }

  fn rget_mut(&mut self, key: VariableReference) -> &mut Self::Out {
    match key {
      VariableReference::Block(block, index) => {
        self.rget_mut(block).variables.get_mut(index).unwrap()
      },
      VariableReference::Argument(function, index) => todo!(),
    }
  }
}
