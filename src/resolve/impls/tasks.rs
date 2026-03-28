pub use crate::resolve::tasks::*;
use crate::lang::ty::Type;

use super::*;

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
    let description = format!(line_dbg!("{}"), prerequisite.explain(lazy));

    let result = tasks.work(description, |tasks| {
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
    let parent = self.dest.reference.parent_module(lazy);

    format!(
      line_dbg!("OverwriteType in {}:\n- dest = {}\n- src  = {}"),
      lazy.describe_module(parent),
      self.dest.print(lazy),
      self.src.print(lazy),
    )
  }

  fn execute(self: Box<Self>, lazy: &mut Lazy, _tasks: &mut Tasks) -> Result<TaskResponse> {
    let span = self.src.get_span(lazy);

    let part = self.dest.reference.parent_module(lazy)
      .add_type_part(self.src, lazy);

    *lazy.rget_mut(self.dest.reference) = Type::Resolved { part, span };

    Ok(TaskResponse::Pop)
  }
}

impl Task for OverwriteExpression {
  fn explain(&self, lazy: &Lazy) -> String {
    let parent = self.dest.0.0.rget_from(lazy).parent;

    format!(
      line_dbg!("OverwriteExpression in {}:\n- dest = {}\n- src  = {}"),
      lazy.describe_module(parent),
      self.dest.print(lazy),
      self.src.print_with(lazy.rget(self.dest.0.0), lazy)
        .collect::<Vec<_>>().join("\n"),
    )
  }

  fn execute(self: Box<Self>, lazy: &mut Lazy, _tasks: &mut Tasks) -> Result<TaskResponse> {
    *lazy.rget_mut(self.dest) = self.src;

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
