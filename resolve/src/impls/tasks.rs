use lang::{Compiler, CompilerPoolStore, module::AddTypePart, tasks::TaskResponse};

pub use crate::tasks::*;
use lang::ty::Type;

use super::*;

impl<C: Compiler + 'static> Task<C> for Subjugate<C> {
  fn explain(&self, store: &C::Store<'_>) -> String {
    let Subjugate { prerequisite, after: original } = self;

    let indent = ">   ";
    let indent = |string: String, count: usize| {
      let spaces = indent.repeat(count);

      let map = string.split('\n')
        .map(|x| format!("{spaces}{x}"))
        .collect::<Vec<_>>();

      map.join("\n")
    };

    let prerequisite_explain = indent(prerequisite.explain(store), 1);
    let original_explain = indent(original.explain(store), 1);

    format!(
      line_dbg!("Executing prerequisite:\n{}\nAnd then:\n{}\n\\"),
      prerequisite_explain.trim_start(),
      original_explain.trim_start(),
    )
  }

  fn execute(self: Box<Self>, store: &mut C::Store<'_>, tasks: &mut Tasks<C>) -> Result<C, TaskResponse<C>> {
    let Self { prerequisite, after } = *self;
    let description = format!(line_dbg!("{}"), prerequisite.explain(store));

    let result = tasks.work(description, |tasks| {
      prerequisite.execute(store, tasks)
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

impl<C: Compiler> Task<C> for OverwriteType<C> {
  fn explain(&self, store: &C::Store<'_>) -> String {
    let parent = self.dest.reference.parent_module(store);

    format!(
      line_dbg!("OverwriteType in {}:\n- dest = {}\n- src  = {}"),
      store.describe_module(parent),
      self.dest.print(store),
      self.src.print(store),
    )
  }

  fn execute(self: Box<Self>, store: &mut C::Store<'_>, _tasks: &mut Tasks<C>) -> Result<C, TaskResponse<C>> {
    let old_span = self.dest.type_of(store)
      .expect("a type to exist here")
      .get_span(store);

    // let span = self.src.get_span(store);
    let span = old_span;

    let part = self.dest.reference.parent_module(store)
      .add_type_part(self.src, store);

    let replace = Type::Resolved { part, span };

    *store.rget_mut(self.dest) = replace;

    Ok(TaskResponse::Pop)
  }
}

impl<C: Compiler> Task<C> for OverwriteExpression<C> {
  fn explain(&self, store: &C::Store<'_>) -> String {
    let parent = self.dest.0.0.rget_from(store).parent;

    format!(
      line_dbg!("OverwriteExpression in {}:\n- dest = {}\n- src  = {}"),
      store.describe_module(parent),
      self.dest.print(store),
      self.src.print_with(store.rget(self.dest.0.0), store)
        .collect::<Vec<_>>().join("\n"),
    )
  }

  fn execute(self: Box<Self>, store: &mut C::Store<'_>, _tasks: &mut Tasks<C>) -> Result<C, TaskResponse<C>> {
    *store.rget_mut(self.dest) = self.src;

    Ok(TaskResponse::Pop)
  }
}

impl<C: Compiler, R: Resolve<C> + Pretty<C, Out = String>> Task<C> for ResolveAsTask<C, R> {
  fn explain(&self, store: &C::Store<'_>) -> String {
    format!(line_dbg!("ResolveAsTask {}"), self.reference.print(store))
  }

  fn execute(self: Box<Self>, store: &mut C::Store<'_>, tasks: &mut Tasks<C>) -> Result<C, TaskResponse<C>> {
    self.reference.resolve(store, tasks)?;

    Ok(TaskResponse::Pop)
  }
}
