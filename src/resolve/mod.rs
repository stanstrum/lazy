mod impls;
mod structure;

mod type_of;
mod tasks;
pub mod verify;

use crate::line_dbg;

use crate::tokenize::token::Span;
use crate::lang::ty::Type;
use crate::lang::reference::{ModuleReference, Reference, Store, TypeReference};
use crate::lang::Lazy;

use type_of::TypeOf;
use tasks::Tasks;

type Result<T> = std::result::Result<T, Box<Error>>;

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
    whence: &'static str,
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

#[derive(Debug)]
pub struct SpecialPair<'a, S: Store<R>, R: Reference<S>>(
  pub &'a R,
  pub &'a S::Out,
);

pub type TypePair<'a, 'b> = SpecialPair<'a, Lazy<'b>, TypeReference>;

trait Resolve {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()>;
}

pub trait Coerce {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()>;
}


pub fn resolve_and_verify(lazy: &mut Lazy, module: ModuleReference) -> Result<()> {
  let mut tasks = Tasks::new();

  tasks.work::<Result<()>>(
    line_dbg!("Resolve global").into(),
    |tasks| loop {
      // Resolve `global` recursively
      module.resolve(lazy, tasks)?;

      // Execute the tasks: typically overwriting unknown values with &mut
      let did_execute = tasks.execute_pass(lazy)?;

      // If no tasks ran, we _should_ be finished resolving
      if !did_execute {
        return Ok(());
      };
    },
  )?;

  tasks.work(
    line_dbg!("Verify global").into(),
    |tasks| verify::program(lazy, module, tasks),
  )?;

  println!(line_dbg!("No further work should be done."));
  assert!(!tasks.execute_pass(lazy)?);

  Ok(())
}
