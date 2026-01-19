use crate::lang::span::GetSpan;
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
    lang::ty::Type::Unresolved { .. } => Err(Error::Unresolved { what: "type", at: ty.get_span() }),
    lang::ty::Type::Intrinsic { .. } => Ok(()),
  }
}

fn verify_function(lazy: &Lazy, function: lang::module::FunctionId) -> Result<(), Error> {
  verify_type(lazy, TypeReference::ReturnTypeOf(function))?;

  let arguments_count = lazy[function].header.arguments.len();
  for index in 0..arguments_count {
    verify_type(lazy, TypeReference::ArgumentOf { function, index })?;
  };

  todo!();

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
