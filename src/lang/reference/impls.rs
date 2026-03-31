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

  fn rget(&self, ExpressionReference(BlockReference(function, _), id): ExpressionReference) -> &Self::Out {
    &self.rget(function)[id]
  }

  fn rget_mut(&mut self, ExpressionReference(BlockReference(function, _), id): ExpressionReference) -> &mut Self::Out {
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

impl<'a> Store<TypeReference> for Lazy<'a> {
  type Out = Type;

  fn rget(&self, reference: TypeReference) -> &Self::Out {
    match reference {
      TypeReference::Part(part) => self.rget(part),
      TypeReference::ReturnTypeOf(function) => {
        &self.rget(function).header.ret_ty
      },
      TypeReference::Alias(alias) => {
        &self.rget(alias).ty
      },
      TypeReference::Variable(VariableReference::Argument(function, index)) => {
        &self.rget(function).header.arguments.get(index).unwrap().ty
      },
      TypeReference::Variable(VariableReference::Block(block, index)) => {
        &self.rget(block).variables.get(index).unwrap().ty
      },
      TypeReference::Expression(expr) => {
        match self.rget(expr) {
          &Expression::Block(block) => &self.rget(block).out,
          | Expression::Literal { out, .. }
          | Expression::Unary { out, .. }
          | Expression::Binary { out, .. }
          | Expression::Unknown { out, .. }
            => out,
          &Expression::Variable { reference, .. } => &self.rget(reference).ty,
        }
      },
      TypeReference::Block(block) => &self.rget(block).out,
    }
  }

  fn rget_mut(&mut self, reference: TypeReference) -> &mut Self::Out {
    match reference {
      TypeReference::Part(part) => self.rget_mut(part),
      TypeReference::ReturnTypeOf(function) => {
        &mut self.rget_mut(function).header.ret_ty
      },
      TypeReference::Alias(alias) => {
        &mut self.rget_mut(alias).ty
      },
      TypeReference::Variable(VariableReference::Argument(function, index)) => {
        &mut self.rget_mut(function).header.arguments.get_mut(index).unwrap().ty
      },
      TypeReference::Variable(VariableReference::Block(block, index)) => {
        &mut self.rget_mut(block).variables.get_mut(index).unwrap().ty
      },
      TypeReference::Expression(expr) => {
        if let &mut Expression::Block(block) = self.rget_mut(expr) {
          let block = self.rget_mut(block);
          return &mut block.out;
        };

        if let Expression::Variable { reference, .. } = self.rget_mut(expr) {
          let reference = *reference;
          let variable = self.rget_mut(reference);
          return &mut variable.ty;
        };
        if let
          | Expression::Unary { out, .. }
          | Expression::Binary { out, .. }
          | Expression::Literal { out, .. }
          | Expression::Unknown { out, .. }
          = self.rget_mut(expr)
        {
          return out;
        };

        unimplemented!()
      },
      TypeReference::Block(block) => {
        &mut self.rget_mut(block).out
      },
    }
  }
}

impl<'a> Store<VariableReference> for Lazy<'a> {
  type Out = Variable;

  fn rget(&self, key: VariableReference) -> &Self::Out {
    match key {
      VariableReference::Block(block, index) => {
        self.rget(block).variables.get(index).unwrap()
      },
      VariableReference::Argument(function, index) => {
        self.rget(function).header.arguments.get(index).unwrap()
      },
    }
  }

  fn rget_mut(&mut self, key: VariableReference) -> &mut Self::Out {
    match key {
      VariableReference::Block(block, index) => {
        self.rget_mut(block).variables.get_mut(index).unwrap()
      },
      VariableReference::Argument(function, index) => {
        self.rget_mut(function).header.arguments.get_mut(index).unwrap()
      },
    }
  }
}
