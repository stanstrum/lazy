use crate::lang::reference::{Store, TypeReference};
use crate::aster::pprint::Pretty;
use crate::lang::span::GetSpan;

use super::*;

pub struct Subjugate {
  prerequisite: Box<dyn Task>,
  original: Box<dyn Task>,
}

pub struct ResolveType {
  pub dest: TypeReference,
  pub value: Type,
}

impl Task for Subjugate {
  fn explain(&self, lazy: &Lazy) -> String {
    let Subjugate { prerequisite, original } = self;

    let prerequisite_explain = (prerequisite).explain(lazy);
    let original_explain = original.explain(lazy);

    let first = std::iter::once(format!("While executing: {prerequisite_explain}"));
    let rest = original_explain
      .split('\n')
      .map(|line| format!("  {line}"));

    first.chain(rest).collect::<Vec<_>>().join("\n")
  }

  fn execute(self: Box<Self>, lazy: &mut Lazy) -> Result<TaskResponse> {
    let replace = match self.original.execute(lazy)? {
      TaskResponse::Replace(replace) => replace,
      TaskResponse::Pop => self.prerequisite,
    };

    Ok(TaskResponse::Replace(replace))
  }
}

impl Task for ResolveType {
  fn explain(&self, lazy: &Lazy) -> String {
    let parent = self.dest.parent_module(lazy);

    format!(
      "ResolveType in {parent}:\n  dest = {dest}\n  value = {value}",
      parent = lazy.describe_module(parent),
      dest = self.dest.print(lazy),
      value = self.value.print(lazy),
    )
  }

  fn execute(self: Box<Self>, lazy: &mut Lazy) -> Result<TaskResponse> {
    let span = self.value.get_span(lazy);

    let part = self.dest.parent_module(lazy)
      .add_type_part(self.value, lazy);

    *lazy.rget_mut(self.dest) = Type::Resolved { part, span };

    Ok(TaskResponse::Pop)
  }
}
