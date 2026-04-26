pub mod impls {
  use lang::Compiler;
  use ::lang::expr::Expression;
  use ::lang::reference::ExpressionReference;

  use super::*;

  pub struct Subjugate<C: Compiler> {
    pub prerequisite: Box<dyn Task<C>>,
    pub after: Box<dyn Task<C>>,
  }

  pub struct OverwriteExpression<C: Compiler> {
    pub dest: ExpressionReference<C>,
    pub src: Expression<C>,
  }

  pub type OverwriteTypeReference = ::lang::ty::OverwriteTypeReference<gluezy::LazyStructures>;
  pub struct OverwriteType {
    pub dest: OverwriteTypeReference,
    pub src: Type,
  }

  pub struct ResolveAsTask<R: Resolve> {
    pub reference: R,
  }
}

use std::rc::Rc;
use std::cell::RefCell;

pub use impls::*;

use super::*;

pub type TaskResponse = ::lang::tasks::TaskResponse<gluezy::LazyStructures>;
pub use ::lang::tasks::{Task, Tasks};

pub struct TaskStatus {
  trace: Rc<RefCell<Vec<String>>>,
}

impl Drop for TaskStatus {
  fn drop(&mut self) {
    self.trace.borrow_mut().pop();
  }
}
