use crate::Lazy;
use ::lang::expr::BlockExpression;
use lang::function::{BlockId, ExprId};
use ::lang::module::TypePartId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleReference(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionReference(pub usize);

pub type TypePartReference = ::lang::reference::TypePartReference<crate::lazy::LazyStructures>;
pub type TypeReference = ::lang::reference::TypeReference<crate::lazy::LazyStructures>;

pub type VariableReference = ::lang::reference::VariableReference<crate::lazy::LazyStructures>;

impl<'pool> Store<ModuleReference> for Lazy<'pool> {
  type Out = crate::lang::module::Module;

  fn rget(&self, ModuleReference(index): ModuleReference) -> &Self::Out {
    self.modules.get(index).unwrap()
  }

  fn rget_mut(&mut self, ModuleReference(index): ModuleReference) -> &mut Self::Out {
    self.modules.get_mut(index).unwrap()
  }
}

impl<'pool> Store<FunctionReference> for Lazy<'pool> {
  type Out = crate::lang::function::Function;

  fn rget(&self, FunctionReference(index): FunctionReference) -> &Self::Out {
    self.functions.get(index).unwrap()
  }

  fn rget_mut(&mut self, FunctionReference(index): FunctionReference) -> &mut Self::Out {
    self.functions.get_mut(index).unwrap()
  }
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
