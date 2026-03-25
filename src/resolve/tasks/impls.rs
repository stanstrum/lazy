use crate::aster::pprint::Pretty;
use crate::lang::reference::{Store, TypeReference};
use crate::lang::span::GetSpan;

use super::*;

pub struct Subjugate {
  pub prerequisite: Box<dyn Task>,
  pub after: Box<dyn Task>,
}

pub struct OverwriteType {
  pub dest: TypeReference,
  pub value: Type,
}

pub struct ResolveAsTask<R: Resolve> {
  pub reference: R,
}

impl Task for Subjugate {
  fn explain(&self, lazy: &Lazy) -> String {
    let Subjugate { prerequisite, after: original } = self;

    let indent = ">   ";
    let indent = |string: String, count: usize| {
      let spaces = indent.repeat(count);

      let map = string.split('\n')
        .map(|x| format!("{spaces}{x}"))
        .collect::<Vec<_>>();

      map.join("\n")
    };

    let prerequisite_explain = indent(prerequisite.explain(lazy), 1);
    let original_explain = indent(original.explain(lazy), 1);

    format!(
      line_dbg!("Executing prerequisite:\n{}\nAnd then:\n{}\n\\"),
      prerequisite_explain.trim_start(),
      original_explain.trim_start(),
    )
  }

  fn execute(self: Box<Self>, lazy: &mut Lazy, tasks: &mut Tasks) -> Result<TaskResponse> {
    let Self { prerequisite, after } = *self;

    let result = task_work(tasks, format!(line_dbg!("{}"), prerequisite.explain(lazy)), |tasks| {
      prerequisite.execute(lazy, tasks)
    });

    let replace = match result? {
      TaskResponse::Replace(replace) => Box::new(Subjugate {
        prerequisite: replace,
        after,
      }),
      TaskResponse::Pop => after,
    };

    Ok(TaskResponse::Replace(replace))
  }
}

impl Task for OverwriteType {
  fn explain(&self, lazy: &Lazy) -> String {
    let parent = self.dest.parent_module(lazy);

    format!(
      line_dbg!("OverwriteType in {}:\n- dest = {}\n- value = {}"),
      lazy.describe_module(parent),
      self.dest.print(lazy),
      self.value.print(lazy),
    )
  }

  fn execute(self: Box<Self>, lazy: &mut Lazy, tasks: &mut Tasks) -> Result<TaskResponse> {
    let span = self.value.get_span(lazy);

    let part = self.dest.parent_module(lazy)
      .add_type_part(self.value, lazy);

    *lazy.rget_mut(self.dest) = Type::Resolved { part, span };

    Ok(TaskResponse::Pop)
  }
}

impl<R: Resolve + Pretty<Out = String>> Task for ResolveAsTask<R> {
  fn explain(&self, lazy: &Lazy) -> String {
    format!(line_dbg!("ResolveAsTask {}"), self.reference.print(lazy))
  }

  fn execute(self: Box<Self>, lazy: &mut Lazy, tasks: &mut Tasks) -> Result<TaskResponse> {
    self.reference.resolve(lazy, tasks)?;

    Ok(TaskResponse::Pop)
  }
}
