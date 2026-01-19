use crate::string_pool::StringPool;

use crate::lang::{self, Lazy};
use crate::resolve::reference::{Reference, TypeReference};

use super::Error;

#[derive(Debug)]
pub enum Task {
  ReplaceType(TypeReference, lang::ty::Type),
}

pub(super) fn execute<'pool>(
  lazy: &mut Lazy<'pool>,
  pool: &'pool StringPool,
  task: Task,
) -> Result<(), Error> {
  match dbg!(task) {
    Task::ReplaceType(reference, replace) => {
      *reference.rget_from_mut(lazy) = replace;
    },
  };

  Ok(())
}
