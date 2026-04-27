pub mod impls {
  use std::marker::PhantomData;

use lang::Compiler;
  use lang::expr::Expression;
  use lang::reference::ExpressionReference;

  use super::*;

  pub struct Subjugate<C: Compiler> {
    pub prerequisite: Box<dyn Task<C>>,
    pub after: Box<dyn Task<C>>,
  }

  pub struct OverwriteExpression<C: Compiler> {
    pub dest: ExpressionReference<C>,
    pub src: Expression<C>,
  }

  pub struct OverwriteType<C: Compiler> {
    pub src: TypeKind<C>,
    pub dest: TypeKind<C>,
  }

  pub struct ResolveAsTask<C: Compiler, R: Resolve<C>> {
    pub reference: R,
    phantom: PhantomData<C>,
  }

  impl<C: Compiler, R: Resolve<C>> ResolveAsTask<C, R> {
    pub fn new(reference: R) -> Self {
      Self {
        reference,
        phantom: Default::default(),
      }
    }
  }
}

use std::rc::Rc;
use std::cell::RefCell;

pub use impls::*;

use super::*;

pub use lang::tasks::{Task, Tasks};

pub struct TaskStatus {
  trace: Rc<RefCell<Vec<String>>>,
}

impl Drop for TaskStatus {
  fn drop(&mut self) {
    self.trace.borrow_mut().pop();
  }
}
