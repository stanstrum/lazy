use crate::lang::{self, Lazy};
use crate::resolve::reference::{Reference, TypeReference};

use super::Error;

#[derive(Debug)]
pub enum Task {
  ReplaceType(TypeReference, lang::ty::Type),
}

pub(super) fn execute<'pool>(
  lazy: &mut Lazy<'pool>,
  task: Task,
) -> Result<(), Error> {
  match task {
    Task::ReplaceType(reference, replace) => {
      *reference.rget_from_mut(lazy) = replace;
    },
  };

  Ok(())
}
