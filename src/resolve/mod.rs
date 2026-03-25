use crate::line_dbg;

use crate::tokenize::token::Span;
use crate::lang::ty::{Intrinsic, Type};
use crate::lang::reference::ModuleReference;
use crate::lang::Lazy;
use crate::resolve::tasks::{OverwriteType, Tasks};

mod tasks;
mod type_of;
mod coerce;
mod structure;
pub mod verify;

mod ty;

#[derive(Debug)]
pub enum Error {
  MissingEntryPoint {
    module_name: String,
    file: ModuleReference,
  },
  UnknownTypeName {
    module_name: String,
    span: Span,
  },
  TypeMismatch {
    a_print: String,
    a_span: Span,
    b_print: String,
    b_span: Span,
  },
  UnresolvedInVerify {
    what: String,
    span: Span,
  },
}

type Result<T> = std::result::Result<T, Box<Error>>;

trait Resolve {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()>;
}

fn task_work<T>(tasks: &mut Tasks, description: String, f: impl FnOnce(&mut Tasks) -> T) -> T {
  let status = tasks.task_work(description);
  println!("{}", tasks.explain(0));
  let result = f(tasks);
  drop(status);
  result
}

pub fn task_resolve(lazy: &mut Lazy, module: ModuleReference) -> Result<()> {
  let mut tasks = Tasks::new();

  task_work::<Result<()>>(&mut tasks,
    line_dbg!("resolve global").into(),
    |tasks| {
      loop {
        module.resolve(lazy, tasks)?;
        let did_execute = tasks.execute_pass(lazy)?;

        if !did_execute {
          break;
        };
      };

      Ok(())
    },
  )?;

  task_work(&mut tasks,
    line_dbg!("Verify global").into(),
    |tasks| {
      verify::program(lazy, module, tasks)
    },
  )?;

  println!(line_dbg!("No further work should be done."));
  assert!(!tasks.execute_pass(lazy)?);

  Ok(())
}
