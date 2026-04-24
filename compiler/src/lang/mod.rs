mod get_span;
mod store;
mod prelude;

pub use prelude::*;

use crate::Lazy;
use crate::lang::expr::BlockExpression;
use lang::function::{BlockId, ExprId};
use crate::lang::module::TypePartId;

pub use lang::reference::{Store, Reference};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionReference(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AliasReference(pub ModuleReference, pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructReference(pub ModuleReference, pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleReference(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockReference(pub FunctionReference, pub BlockId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpressionReference(pub BlockReference, pub ExprId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypePartReference(pub ModuleReference, pub TypePartId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeReference {
  Alias(AliasReference),
  Part(TypePartReference),
  Expression(ExpressionReference),
  Block(BlockReference),
  ReturnTypeOf(FunctionReference),
  Variable(VariableReference),
  StructMember(StructReference, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableReference {
  Block(BlockReference, usize),
  Argument(FunctionReference, usize),
}

impl FunctionReference {
  pub fn body(&self) -> BlockReference {
    BlockReference(*self, BlockId::body_id())
  }

  // pub fn get_body<'a>(&self, lazy: &'a Lazy) -> &'a BlockExpression {
  //   self.body(lazy).rget_from(lazy)
  // }

  pub fn get_body_mut<'a>(&self, lazy: &'a mut Lazy) -> &'a mut BlockExpression {
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

impl TypeReference {
  pub fn parent_module(&self, lazy: &Lazy) -> ModuleReference {
    match self {
      &TypeReference::Alias(AliasReference(module_reference, _))
        => module_reference,
      &TypeReference::Part(TypePartReference(module_reference, _))
        => module_reference,
      | TypeReference::Expression(ExpressionReference(BlockReference(function_reference, _), _))
      | TypeReference::ReturnTypeOf(function_reference) => {
      let function = function_reference.rget_from(lazy);
        function.parent
      },
      TypeReference::Variable(v) => lazy.rget(v.parent()).parent,
      TypeReference::Block(BlockReference(function, _)) => {
        function.rget_from(lazy).parent
      },
      &TypeReference::StructMember(StructReference(parent, _), _) => parent,
    }
  }
}

impl VariableReference {
  pub fn parent(&self) -> FunctionReference {
    match self {
      VariableReference::Block(block_reference, _) => block_reference.0,
      VariableReference::Argument(function_reference, _) => *function_reference,
    }
  }
}
