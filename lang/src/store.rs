use crate::Compiler;
use crate::expr::Expression;
use crate::reference::{AliasReference, BlockReference, ExpressionReference, Store, StructReference, TypePartId, TypePartReference, TypeReference, VariableReference};
use crate::ty::TypeKind;

impl<'pool, C: Compiler> Store<AliasReference<C>> for C::Store<'pool> {
  type Out = crate::module::TypeAlias<C>;

  fn rget(&self, AliasReference(module, index): AliasReference<C>) -> &Self::Out {
    self.rget(module).aliases.get(index).unwrap()
  }

  fn rget_mut(&mut self, AliasReference(module, index): AliasReference<C>) -> &mut Self::Out {
    self.rget_mut(module).aliases.get_mut(index).unwrap()
  }
}

impl<'pool, C: Compiler> Store<StructReference<C>> for C::Store<'pool> {
  type Out = crate::module::Struct<C>;

  fn rget(&self, StructReference(module, index): StructReference<C>) -> &Self::Out {
    self.rget(module).structs.get(index).unwrap()
  }

  fn rget_mut(&mut self, StructReference(module, index): StructReference<C>) -> &mut Self::Out {
    self.rget_mut(module).structs.get_mut(index).unwrap()
  }
}

impl<'pool, C: Compiler> Store<BlockReference<C>> for C::Store<'pool> {
  type Out = crate::expr::BlockExpression<C>;

  fn rget(&self, BlockReference(function, id): BlockReference<C>) -> &Self::Out {
    &self.rget(function)[id]
  }

  fn rget_mut(&mut self, BlockReference(function, id): BlockReference<C>) -> &mut Self::Out {
    &mut self.rget_mut(function)[id]
  }
}

impl<'pool, C: Compiler> Store<ExpressionReference<C>> for C::Store<'pool> {
  type Out = crate::expr::Expression<C>;

  fn rget(&self, ExpressionReference(BlockReference(function, _), id): ExpressionReference<C>) -> &Self::Out {
    &self.rget(function)[id]
  }

  fn rget_mut(&mut self, ExpressionReference(BlockReference(function, _), id): ExpressionReference<C>) -> &mut Self::Out {
    &mut self.rget_mut(function)[id]
  }
}

impl<'pool, C: Compiler> Store<TypePartReference<C>> for C::Store<'pool> {
  type Out = crate::ty::TypeKind<C>;

  fn rget(&self, TypePartReference(module, TypePartId(index)): TypePartReference<C>) -> &Self::Out {
    self.rget(module).type_parts.get(index).unwrap()
  }

  fn rget_mut(&mut self, TypePartReference(module, TypePartId(index)): TypePartReference<C>) -> &mut Self::Out {
    self.rget_mut(module).type_parts.get_mut(index).unwrap()
  }
}

impl<'pool, C: Compiler> Store<TypeReference<C>> for C::Store<'pool> {
  type Out = crate::ty::TypeKind<C>;

  fn rget(&self, reference: TypeReference<C>) -> &Self::Out {
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
          | Expression::StructInitializer { ty: out, .. }
            => out,
          &Expression::Variable { reference, .. } => &self.rget(reference).ty,
        }
      },
      TypeReference::Block(block) => &self.rget(block).out,
      TypeReference::StructMember(struct_reference, id) => {
        &self.rget(struct_reference).members.get(id).unwrap().ty
      },
    }
  }

  fn rget_mut(&mut self, reference: TypeReference<C>) -> &mut Self::Out {
    match reference {
      TypeReference::Part(part) => self.rget_mut(part),
      TypeReference::ReturnTypeOf(function) => {
        &mut self.rget_mut(function).header.ret_ty
      },
      TypeReference::StructMember(struct_reference, id) => {
        &mut self.rget_mut(struct_reference).members.get_mut(id).unwrap().ty
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
          | Expression::StructInitializer { ty: out, .. }
          = self.rget_mut(expr)
        {
          return out;
        };

        unimplemented!("get `out` type for expression")
      },
      TypeReference::Block(block) => {
        &mut self.rget_mut(block).out
      },
    }
  }
}

impl<'pool, C: Compiler> Store<VariableReference<C>> for C::Store<'pool> {
  type Out = crate::expr::Variable<C>;

  fn rget(&self, key: VariableReference<C>) -> &Self::Out {
    match key {
      VariableReference::Block(block, index) => {
        self.rget(block).variables.get(index).unwrap()
      },
      VariableReference::Argument(function, index) => {
        self.rget(function).header.arguments.get(index).unwrap()
      },
    }
  }

  fn rget_mut(&mut self, key: VariableReference<C>) -> &mut Self::Out {
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
