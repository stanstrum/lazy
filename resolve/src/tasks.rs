use lang::error::ResolveError;

use crate::{Compiler, Resolver};

pub trait Task<C: Compiler> {
  fn explain<'store, 'pool, 'tasks>(&self, resolver: &'store Resolver<'store, 'pool, 'tasks, C>) -> String;
  fn execute<'store, 'pool, 'tasks>(self: Box<Self>, resolver: &Resolver<'store, 'pool, 'tasks, C>) -> Result<TaskResponse<C>, Box<ResolveError<C>>>;
}

impl<C: Compiler> Task<C> for Box<dyn Task<C>> {
  fn explain<'store, 'pool, 'tasks>(&self, resolver: &'store Resolver<'store, 'pool, 'tasks, C>) -> String {
    todo!()
  }

  fn execute<'store, 'pool, 'tasks>(self: Box<Self>, resolver: &Resolver<'store, 'pool, 'tasks, C>) -> Result<TaskResponse<C>, Box<ResolveError<C>>> {
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
