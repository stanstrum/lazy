pub mod reference;
pub mod verify;
mod task;
mod r#typeof;

use std::collections::VecDeque;

use crate::resolve::reference::ExpressionReference;
use crate::{error::*, line_dbg};
use crate::lang::{self, Lazy};
use crate::lang::span::GetSpan;

use task::Task;
use reference::{Reference, TypeReference};

#[derive(Debug)]
pub enum Error {
  Verify(verify::Error),
}

fn resolve_type<'pool>(
  lazy: &mut Lazy<'pool>,
  reference: TypeReference,
  tasks: &mut VecDeque<Task>,
) -> Result<bool, Error> {
  match reference.rget_from(lazy) {
    lang::ty::Type::Unresolved { qualified, .. } if qualified.parts.len() == 1 => {
      let first_id = qualified.parts.first().unwrap().id;
      let first = lazy.pool.get(first_id).collect::<String>();

      if let Some(kind) = lang::ty::Intrinsic::try_from_str(&first) {
        tasks.push_back(Task::ReplaceType(reference, lang::ty::Type::Intrinsic {
          kind,
          span: qualified.span,
        }));

        Ok(true)
      } else {
        Ok(false)
      }
    },
    lang::ty::Type::Unresolved { .. } => todo!(),
    lang::ty::Type::Intrinsic { .. } => Ok(false),
    lang::ty::Type::Deferred(reference) => {
      resolve_type(lazy, *reference, tasks)
    },
    lang::ty::Type::WeakInteger { .. } => Ok(false),
    lang::ty::Type::WeakFloat { .. } => Ok(false),
  }
}

fn resolve_expr<'pool>(
  lazy: &mut Lazy<'pool>,
  reference: ExpressionReference,
  tasks: &mut VecDeque<Task>,
) -> Result<bool, Error> {
  match reference.rget_from(lazy) {
    lang::expr::Expression::BlockExpression(block) => resolve_block_expr(lazy, reference.function, *block, tasks),
    lang::expr::Expression::Literal { .. } => Ok(false),
  }
}

fn resolve_block_expr<'pool>(
  lazy: &mut Lazy<'pool>,
  function: lang::module::FunctionId,
  block: lang::function::BlockId,
  tasks: &mut VecDeque<Task>,
) -> Result<bool, Error> {
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

  unsafe {
    static mut SHOWN: bool = false;

    if !SHOWN {
      print_message(lazy, PrintableMesage {
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

      SHOWN = true;
    };
  };

  Ok(did_work)
}

fn resolve_function<'pool>(
  lazy: &mut Lazy<'pool>,
  function: lang::module::FunctionId,
  tasks: &mut VecDeque<Task>,
) -> Result<bool, Error> {
  let mut did_work = false;

  did_work |= resolve_type(lazy, TypeReference::ReturnTypeOf(function), tasks)?;

  let arguments_count = lazy[function].header.arguments.len();
  for index in 0..arguments_count {
    let reference = TypeReference::ArgumentOf { function, index };
    did_work |= resolve_type(lazy, reference, tasks)?;
  };

  did_work |= resolve_block_expr(lazy, function, lazy[function].body, tasks)?;

  Ok(did_work)
}

fn resolve_module<'pool>(
  lazy: &mut Lazy<'pool>,
  id: lang::module::ModuleId,
  tasks: &mut VecDeque<Task>
) -> Result<bool, Error> {
  let mut did_work = false;

  for func_id in lazy[id].functions.clone() {
    did_work |= resolve_function(lazy, func_id, tasks)?;
  };

  for child_id in lazy[id].modules.clone() {
    did_work |= resolve_module(lazy, child_id, tasks)?;
  };

  Ok(did_work)
}

pub fn resolve<'pool>(
  lazy: &mut Lazy<'pool>,
  entry: lang::module::ModuleId,
) -> Result<(), Error> {
  let mut tasks = VecDeque::new();

  while resolve_module(lazy, entry, &mut tasks)? {
    while let Some(task) = tasks.pop_front() {
      task::execute(lazy, task)?;
    };
  };

  if let Err(err) = verify::verify_module(lazy, entry) {
    return Err(Error::Verify(err));
  };

  Ok(())
}
