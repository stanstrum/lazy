use crate::lang::{self, Lazy};
use crate::resolve::reference::{Reference, TypeReference};

use super::Error;

#[derive(Debug)]
pub enum Task {
  ReplaceType(TypeReference, lang::ty::Type),
  ResolveQualified(TypeReference, TypeReference),
}

pub(super) fn execute<'pool>(
  lazy: &mut Lazy<'pool>,
  task: Task,
) -> Result<(), Box<Error>> {
  match task {
    Task::ReplaceType(dest, replace) => {
      *dest.rget_from_mut(lazy) = replace;
    },
    Task::ResolveQualified(dest, reference) => {
      let dest = dest.rget_from_mut(lazy);

      *dest = lang::ty::Type::Deferred {
        original: Box::new(dest.to_owned()),
        reference
      };
    },
  };

  Ok(())
}
