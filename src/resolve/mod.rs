use crate::line_dbg;

use crate::tokenize::token::Span;
use crate::lang::ty::{Intrinsic, Type};
use crate::lang::reference::ModuleReference;
use crate::lang::Lazy;
use crate::resolve::tasks::{ResolveType, Tasks};
use crate::resolve::ty::ResolvedTypePair;

mod tasks;
mod type_of;
mod coerce;
mod structure;
// pub mod verify;

mod ty;

#[derive(Debug)]
pub enum Error {
  UnknownTypeName {
    module_name: String,
    span: Span,
  },
}

type Result<T> = std::result::Result<T, Box<Error>>;

trait Resolve {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()>;
}

pub fn task_resolve(lazy: &mut Lazy, module: ModuleReference) -> Result<()> {
  let mut tasks = Tasks::new();

  tasks.task_status(line_dbg!("resolve global").into());

  loop {
    module.resolve(lazy, &mut tasks)?;
    let did_execute = tasks.execute_pass(lazy)?;

    if !did_execute {
      break;
    };
  };

  Ok(())
}
