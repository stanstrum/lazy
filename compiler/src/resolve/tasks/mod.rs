mod impls;

use std::rc::Rc;
use std::collections::VecDeque;
use std::cell::RefCell;

pub use impls::*;

use super::*;

pub type TaskResponse = ::lang::tasks::TaskResponse<crate::LazyStructures>;
pub use ::lang::tasks::{Task, Tasks};

pub struct TaskStatus {
  trace: Rc<RefCell<Vec<String>>>,
}

impl Drop for TaskStatus {
  fn drop(&mut self) {
    self.trace.borrow_mut().pop();
  }
}
