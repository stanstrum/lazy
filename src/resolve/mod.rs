mod reference;
mod verify;
mod task;

use std::collections::VecDeque;

use crate::lang::{self, Lazy};
use crate::string_pool::StringPool;

use task::Task;
use reference::{Reference, TypeReference};

#[derive(Debug)]
pub enum Error {
  Verify(verify::Error),
}

fn resolve_type<'pool>(
  lazy: &mut Lazy<'pool>,
  pool: &'pool StringPool,
  reference: TypeReference,
  tasks: &mut VecDeque<Task>,
) -> Result<(), Error> {
  match reference.rget_from(lazy) {
    lang::ty::Type::Unresolved { qualified, .. } if qualified.parts.len() == 1 => {
      let first_id = qualified.parts.first().unwrap();
      let first = lazy.pool.get(*first_id).collect::<String>();
      if let Some(kind) = lang::ty::Intrinsic::try_from_str(&first) {
        tasks.push_back(Task::ReplaceType(reference, lang::ty::Type::Intrinsic {
          kind,
          span: qualified.span,
        }));
      };
    },
    lang::ty::Type::Unresolved { .. } => todo!(),
    lang::ty::Type::Intrinsic { .. } => { /* do nothing */ },
  };

  Ok(())
}

fn resolve_function<'pool>(
  lazy: &mut Lazy<'pool>,
  pool: &'pool StringPool,
  function: lang::module::FunctionId,
  tasks: &mut VecDeque<Task>,
) -> Result<(), Error> {
  resolve_type(lazy, pool, TypeReference::ReturnTypeOf(function), tasks)?;

  let arguments_count = lazy[function].header.arguments.len();
  for index in 0..arguments_count {
    let reference = TypeReference::ArgumentOf { function, index };
    resolve_type(lazy, pool, reference, tasks)?;
  };

  Ok(())
}

fn resolve_module<'pool>(
  lazy: &mut Lazy<'pool>,
  pool: &'pool StringPool,
  id: lang::module::ModuleId,
  tasks: &mut VecDeque<Task>
) -> Result<(), Error> {
  for func_id in lazy[id].functions.clone() {
    resolve_function(lazy, pool, func_id, tasks)?;
  };

  for child_id in lazy[id].modules.clone() {
    resolve_module(lazy, pool, child_id, tasks)?;
  };

  Ok(())
}

pub fn resolve<'pool>(
  lazy: &mut Lazy<'pool>,
  pool: &'pool StringPool,
  entry: lang::module::ModuleId,
) -> Result<(), Error> {
  let mut tasks = VecDeque::new();

  loop {
    resolve_module(lazy, pool, entry, &mut tasks)?;

    if tasks.is_empty() {
      break;
    };

    while let Some(task) = tasks.pop_front() {
      task::execute(lazy, pool, task)?;
    };
  };

  if let Err(err) = verify::verify_module(lazy, entry) {
    return Err(Error::Verify(err));
  };

  Ok(())
}
