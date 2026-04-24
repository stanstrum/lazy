use crate::Lazy;
use ::lang::expr::BlockExpression;
use lang::function::{BlockId, ExprId};
use ::lang::module::TypePartId;

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

pub type TypePartReference = ::lang::reference::TypePartReference<crate::lazy::LazyStructures>;
pub type TypeReference = ::lang::reference::TypeReference<crate::lazy::LazyStructures>;

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
