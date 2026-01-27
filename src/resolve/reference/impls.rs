use crate::lang::Lazy;
use crate::lang::expr::Expression;
use crate::lang::module::{FunctionId, Module, ModuleId};
use crate::lang::function::{ExprId, Function};
use crate::lang::ty::Type;

use super::*;

#[derive(Debug, Clone)]
pub enum TypeReference {
  ReturnTypeOf(lang::module::FunctionId),
  ArgumentOf {
    function: lang::module::FunctionId,
    index: usize,
  },
  Dereference(Box<TypeReference>),
  // Should this be an AliasId or maybe AliasReference?
  Alias {
    module: ModuleId,
    index: usize,
  },
}

#[derive(Debug, Clone, Copy)]
pub struct ExpressionReference {
  pub function: FunctionId,
  pub index: ExprId,
}

impl<'a> Reference<'a> for ModuleId {
  type Parent<'b> = Lazy<'b>;
  type Out = Module;

  fn rget_from(&self, parent: &'a Self::Parent<'_>) -> &'a Self::Out {
    &parent[*self]
  }

  fn rget_from_mut(&self, parent: &'a mut Self::Parent<'_>) -> &'a mut Self::Out {
    &mut parent[*self]
  }
}

impl<'a> Reference<'a> for FunctionId {
  type Parent<'b> = Lazy<'b>;
  type Out = Function;

  fn rget_from(&self, parent: &'a Self::Parent<'_>) -> &'a Self::Out {
    &parent[*self]
  }

  fn rget_from_mut(&self, parent: &'a mut Self::Parent<'_>) -> &'a mut Self::Out {
    &mut parent[*self]
  }
}

impl<'a> Reference<'a> for TypeReference {
  type Parent<'b> = Lazy<'b>;
  type Out = Type;

  fn rget_from(&self, parent: &'a Self::Parent<'_>) -> &'a Self::Out {
    match self {
      TypeReference::ReturnTypeOf(function) => {
        &parent
          .rget(function)
          .header
          .ret_ty
      },
      TypeReference::ArgumentOf { function, index } => {
        &parent
          .rget(function)
          .header
          .arguments
          .get(*index).unwrap()
          .ty
      },
      TypeReference::Dereference(inner) => {
        let Type::ReferenceTo { ty, .. } = inner.rget_from(parent) else {
          unreachable!();
        };

        ty.as_ref()
      },
      TypeReference::Alias { module, index } => {
        &parent[*module].aliases.get(*index).unwrap().ty
      },
    }
  }

  fn rget_from_mut(&self, parent: &'a mut Self::Parent<'_>) -> &'a mut Self::Out {
    match self {
      TypeReference::ReturnTypeOf(function) => {
        &mut parent
          .rget_mut(function)
          .header
          .ret_ty
      },
      TypeReference::ArgumentOf { function, index } => {
        &mut parent
          .rget_mut(function)
          .header
          .arguments
          .get_mut(*index).unwrap()
          .ty
      },
      TypeReference::Dereference(inner) => {
        let Type::ReferenceTo { ty, .. } = inner.rget_from_mut(parent) else {
          unreachable!();
        };

        ty.as_mut()
      },
      TypeReference::Alias { module, index } => {
        &mut parent[*module].aliases.get_mut(*index).unwrap().ty
      },
    }
  }
}

impl<'a> Reference<'a> for ExpressionReference {
  type Parent<'b> = Lazy<'b>;
  type Out = Expression;

  fn rget_from(&self, parent: &'a Self::Parent<'_>) -> &'a Self::Out {
    &parent[self.function][self.index]
  }

  fn rget_from_mut(&self, parent: &'a mut Self::Parent<'_>) -> &'a mut Self::Out {
    &mut parent[self.function][self.index]
  }
}
