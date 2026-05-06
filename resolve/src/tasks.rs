use std::collections::VecDeque;
use std::cell::RefCell;

use lang::error::ResolveError;

use crate::{Compiler, Resolver};

pub struct Tasks<C: Compiler> {
  pub(crate) tasks: RefCell<VecDeque<Box<dyn Task<C>>>>,
  pub(crate) trace: RefCell<Vec<String>>,
}

pub trait Task<C: Compiler> {
  fn explain<'store, 'pool, 'tasks>(&self, resolver: &'store Resolver<'store, 'pool, 'tasks, C>) -> String;
  fn execute<'store, 'pool, 'tasks>(self: Box<Self>, resolver: &Resolver<'store, 'pool, 'tasks, C>) -> Result<TaskResponse<C>, Box<ResolveError<C>>>;
}

impl<C: Compiler> Tasks<C> {
  pub(crate) fn new() -> Self {
    Self {
      tasks: RefCell::new(VecDeque::new()),
      trace: RefCell::new(vec![]),
    }
  }

  // pub fn push(&mut self, task: impl Task<C> + 'static, _source: &'static str) {
  //   // #[cfg(debug_assertions)] println!(line_dbg!("push from {}"), _source);
  //   self.tasks.borrow_mut().push_back(Box::new(task));
  // }
}

impl<C: Compiler> Task<C> for Box<dyn Task<C>> {
  fn explain<'store, 'pool, 'tasks>(&self, _resolver: &'store Resolver<'store, 'pool, 'tasks, C>) -> String {
    todo!()
  }

  fn execute<'store, 'pool, 'tasks>(self: Box<Self>, _resolver: &Resolver<'store, 'pool, 'tasks, C>) -> Result<TaskResponse<C>, Box<ResolveError<C>>> {
    todo!()
  }
}

// impl<C: Compiler> Task<C> for Box<dyn Task<C>> {
//   fn explain(&self, store: &C::Store<'_>) -> String {
//     self.as_ref().explain(store)
//   }

//   fn execute(self: Box<Self>, store: &mut C::Store<'_>, tasks: &impl Tasks<C>) -> Result<TaskResponse<C>, Box<ResolveError<C>>> {
//     (*self).execute(store, tasks)
//   }
// }

pub enum TaskResponse<C: Compiler> {
  /// Pop this task -- it's done
  Pop,

  /// Replace this task -- its function/requirements/parameters have changed
  Replace(Box<dyn Task<C>>),
}
