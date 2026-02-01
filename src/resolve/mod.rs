pub mod reference;
pub mod verify;
mod task;
mod r#typeof;
mod coerce;

use std::collections::VecDeque;

use crate::resolve::coerce::IsResolved;
use crate::resolve::reference::{BlockReference, ExpressionReference};
use crate::resolve::task::{DoTask, Tasks};
use crate::tokenize::token::Span;
use crate::line_dbg;
use crate::lang::{self, Lazy};
use crate::lang::span::GetSpan;

use reference::{Reference, TypeReference};

#[derive(Debug)]
pub enum Error {
  Incompatible {
    what: String,
    what_span: Span,
    to: String,
    to_span: Span,
  },
  Unresolved {
    what: &'static str,
    at: Span,
  },
}

fn resolve_type<'pool>(
  lazy: &mut Lazy<'pool>,
  reference: &TypeReference,
  tasks: &mut Tasks,
) -> Result<bool, Box<Error>> {
  match reference.rget_from(lazy) {
    lang::ty::Type::Reference(reference) => resolve_type(lazy, &reference.to_owned(), tasks),
    lang::ty::Type::Unresolved { qualified, module } => {
      if !qualified.implicit && qualified.parts.len() == 1 {
        let first_id = qualified.parts.first().unwrap().id;
        let first = lazy.pool.get(first_id).collect::<String>();

        if let Some(kind) = lang::ty::Intrinsic::try_from_str(&first) {
          tasks.push_back(task::ReplaceType {
            dest: reference.to_owned(),
            src: lang::ty::Type::Intrinsic {
              kind,
              span: qualified.span,
            },
          }.into_task());

          return Ok(true);
        };
      };

      let mut here = *module;
      let (last, rest) = qualified.parts.split_last().unwrap();
      'next: for part in rest {
        for &candidate in lazy[here].modules.iter() {
          if lazy[candidate].name == part.id {
            here = candidate;
            continue 'next;
          };
        };

        return Ok(false);
      };

      if let Some(index) = lazy[here].aliases.iter()
        .position(|alias| alias.name.id == last.id)
      {
        tasks.push_back(
          task::ResolveQualified {
            dest: reference.to_owned(),
            reference: TypeReference::Alias {
              module: here,
              index,
            },
          }.into_task()
        );

        return Ok(true);
      };

      Ok(false)
    },
    lang::ty::Type::Intrinsic { .. } => Ok(false),
    lang::ty::Type::Resolved { reference, .. } => {
      resolve_type(lazy, &reference.to_owned(), tasks)
    },
    lang::ty::Type::WeakInteger { .. } => Ok(false),
    lang::ty::Type::WeakFloat { .. } => Ok(false),
    lang::ty::Type::ReferenceTo { .. } => {
      resolve_type(lazy, &TypeReference::Dereference(Box::new(reference.to_owned())), tasks)
    },
    _ => todo!(),
  }
}

fn resolve_expr<'pool>(
  lazy: &mut Lazy<'pool>,
  reference: ExpressionReference,
  tasks: &mut Tasks,
) -> Result<bool, Box<Error>> {
  match reference.rget_from(lazy) {
    lang::expr::Expression::BlockExpression(block) => resolve_block_expr(lazy, reference.function, *block, tasks),
    lang::expr::Expression::Literal { .. } => Ok(false),
  }
}

fn resolve_block_expr<'pool>(
  lazy: &mut Lazy<'pool>,
  function: lang::module::FunctionId,
  block: lang::function::BlockId,
  tasks: &mut Tasks,
) -> Result<bool, Box<Error>> {
  let mut did_work = false;
  // let ret_ty = lang::ty::Type::Deferred(TypeReference::ReturnTypeOf(function));

  for index in lazy[function][block].children.clone() {
    let reference = ExpressionReference { function, index };
    did_work |= resolve_expr(lazy, reference, tasks)?;
  };

  let block = &lazy[function][block];
  let block_span = block.span;
  let children = &block.children;
  let span = children.last()
    .map(|&child| {
      let parent = &lazy[function];
      parent[child].get_span(parent)
    }).unwrap_or(block_span);
  let mut range = lazy[function].span.to_owned();
  range.extend(span);

  // coerce::coerce(lazy, what, to)
  print_message(lazy, PrintableMessage {
    level: crate::error::Level::Warn,
    force: false,
    description: line_dbg!("stub: check for last-return").into(),
    contents: crate::error::MessageContents::WithinSource {
      range,
      sections: vec![
        MessageSection {
          text: "in this function".into(),
          span: lazy[function].header.name.span,
        },
        MessageSection {
          text: "here".into(),
          span,
        }
      ],
    },
  });

  Ok(did_work)
}

fn resolve_function<'pool>(
  lazy: &mut Lazy<'pool>,
  function: lang::module::FunctionId,
  tasks: &mut Tasks,
) -> Result<bool, Box<Error>> {
  let mut did_work = false;

  did_work |= resolve_type(lazy, &TypeReference::ReturnTypeOf(function), tasks)?;

  let arguments_count = lazy[function].header.arguments.len();
  for index in 0..arguments_count {
    let reference = TypeReference::ArgumentOf { function, index };
    did_work |= resolve_type(lazy, &reference, tasks)?;
  };

  did_work |= resolve_block_expr(lazy, function, lazy[function].body, tasks)?;

  Ok(did_work)
}

fn resolve_module<'pool>(
  lazy: &mut Lazy<'pool>,
  module: lang::module::ModuleId,
  tasks: &mut Tasks
) -> Result<bool, Box<Error>> {
  let mut did_work = false;

  for index in 0..lazy[module].aliases.len() {
    did_work |= resolve_type(lazy, &TypeReference::Alias { module, index }, tasks)?;
  };

  for func_id in lazy[module].functions.clone() {
    did_work |= resolve_function(lazy, func_id, tasks)?;
  };

  for child_id in lazy[module].modules.clone() {
    did_work |= resolve_module(lazy, child_id, tasks)?;
  };

  Ok(did_work)
}

pub fn resolve<'pool>(
  lazy: &mut Lazy<'pool>,
  entry: lang::module::ModuleId,
) -> Result<(), Box<Error>> {
  let mut tasks: Tasks = VecDeque::new();

  while resolve_module(lazy, entry, &mut tasks)? {
    while let Some(task) = tasks.pop_front() {
      task.this.apply(lazy, &mut tasks)?;

      for this in task.and_then.into_iter() {
        tasks.push_back(task::Task { this, and_then: vec![] });
      };
    };
  };

  verify::verify_module(lazy, entry)?;

  Ok(())
}
