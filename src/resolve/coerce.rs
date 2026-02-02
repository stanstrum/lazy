use crate::aster::pprint::Pretty;
use crate::lang::expr::BlockExpression;
use crate::lang::ty::Type;

use super::*;

pub(super) fn assert_assignable(lazy: &Lazy, what: &TypeReference, ty: &lang::ty::Type) -> Result<(), Box<Error>> {
  match r#typeof::is_assignable(lazy, what, ty)? {
    Some(true) => Ok(()),
    Some(false) => Err(Box::new(Error::Incompatible {
      what: what.print(lazy),
      what_span: what.get_span(lazy),
      to: ty.print(lazy),
      to_span: ty.get_span(lazy),
    })),
    None => Err(Box::new(Error::Unresolved {
      what: "type",
      at: ty.get_span(lazy),
    })),
  }
}

pub(super) trait IsResolved {
  fn is_resolved(&self, lazy: &Lazy) -> Result<bool, Box<Error>>;
}

impl IsResolved for TypeReference {
  fn is_resolved(&self, lazy: &Lazy) -> Result<bool, Box<Error>> {
    self.rget_from(lazy).is_resolved(lazy)
  }
}

impl IsResolved for Type {
  fn is_resolved(&self, lazy: &Lazy) -> Result<bool, Box<Error>> {
    match self {
      Type::Reference(reference) => reference.is_resolved(lazy),
      Type::Intrinsic { kind, span } => Ok(true),
      Type::Resolved { reference, .. } => reference.is_resolved(lazy),
      | Type::ReferenceTo { ty, .. }
      | Type::UnsizedArrayOf { ty, .. }
      | Type::SizedArrayOf { ty, .. } => ty.is_resolved(lazy),
      Type::Expression(expression) => Ok(r#typeof::type_of(lazy, expression)?.is_some()),
      _ => todo!(),
    }
  }
}

impl IsResolved for BlockExpression {
  fn is_resolved(&self, lazy: &Lazy) -> Result<bool, Box<Error>> {
    self.out.is_resolved(lazy)
  }
}

pub(super) trait Coerce<R: for<'a> Reference<'a, Out = Self>>: std::fmt::Debug {
  fn coerce(&self, lazy: &Lazy, reference: &R, to: &TypeReference, tasks: &mut Tasks) -> Result<(), Box<Error>>;
}

impl Coerce<TypeReference> for Type {
  fn coerce(&self, lazy: &Lazy, reference: &TypeReference, to: &TypeReference, tasks: &mut Tasks) -> Result<(), Box<Error>> {
    let ty = reference.rget_from(lazy);

    assert_assignable(lazy, reference, ty)?;

    tasks.push_back(task::ReplaceType {
      dest: reference.to_owned(),
      src: ty.to_owned(),
    }.into_task());

    Ok(())
  }
}

impl Coerce<BlockReference> for BlockExpression {
  fn coerce(&self, lazy: &Lazy, reference: &BlockReference, to: &TypeReference, tasks: &mut Tasks) -> Result<(), Box<Error>> {
    self.out.coerce(lazy, &TypeReference::Block(reference.to_owned()), to, tasks)
  }
}
