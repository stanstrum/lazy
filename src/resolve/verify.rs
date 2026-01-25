use crate::error::{Level, MessageContents, MessageSection, PrintableMesage, print_message};
use crate::lang::span::GetSpan;
use crate::line_dbg;
use crate::tokenize::token::Span;
use crate::resolve::reference::{Reference, TypeReference};
use crate::lang::{self, Lazy};

#[derive(Debug)]
pub enum Error {
  Unresolved {
    what: &'static str,
    at: Span,
  },
}

fn verify_type(lazy: &Lazy, reference: TypeReference) -> Result<(), Error> {
  let ty = reference.rget_from(lazy);

  match ty {
    lang::ty::Type::Unresolved { .. } => Err(Error::Unresolved {
      what: line_dbg!("type"),
      at: ty.get_span(lazy),
    }),
    lang::ty::Type::Intrinsic { .. } => Ok(()),
    lang::ty::Type::Deferred(reference) => verify_type(lazy, *reference),
    lang::ty::Type::WeakInteger { span } => Err(Error::Unresolved {
      what: line_dbg!("integer"),
      at: *span,
    }),
    lang::ty::Type::WeakFloat { span } => Err(Error::Unresolved {
      what: line_dbg!("float"),
      at: *span,
    }),
  }
}

fn verify_expr(lazy: &Lazy, function: lang::module::FunctionId, block: lang::function::BlockId, index: usize) -> Result<(), Error> {
  match &lazy[function][block].children[index] {
    lang::expr::Expression::BlockExpression(block) => verify_block(lazy, function, *block),
    lang::expr::Expression::Literal { .. } => Ok(()),
  }
}

fn verify_block(lazy: &Lazy, function: lang::module::FunctionId, block: lang::function::BlockId) -> Result<(), Error> {
  let children_count = lazy[function][block].children.len();

  for index in 0..children_count {
    verify_expr(lazy, function, block, index)?;
  };

  Ok(())
}

fn verify_function(lazy: &Lazy, function: lang::module::FunctionId) -> Result<(), Error> {
  verify_type(lazy, TypeReference::ReturnTypeOf(function))?;

  let arguments_count = lazy[function].header.arguments.len();
  for index in 0..arguments_count {
    verify_type(lazy, TypeReference::ArgumentOf { function, index })?;
  };

  let function_ref = &lazy[function];
  let body = function_ref.body;
  verify_block(lazy, function, body)?;

  let last_span = function_ref[body].children.last()
    .map(|child| child.get_span(function_ref))
    .unwrap_or(function_ref[body].span);

  print_message(lazy, PrintableMesage {
    level: Level::Warn,
    force: false,
    description: line_dbg!("stub: verify return-last").into(),
    contents: MessageContents::WithinSource {
      range: last_span,
      sections: vec![MessageSection {
        text: "here".into(),
        span: last_span,
      }],
    },
  });

  Ok(())
}

pub(super) fn verify_module(lazy: &Lazy, id: lang::module::ModuleId) -> Result<(), Error> {
  for &id in lazy[id].modules.iter() {
    verify_module(lazy, id)?;
  };

  for &id in lazy[id].functions.iter() {
    verify_function(lazy, id)?;
  };

  Ok(())
}
