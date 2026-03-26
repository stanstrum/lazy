use crate::lang::span::GetSpan;
use crate::resolve::type_of::TypeOf;
use crate::lang::ty::{Qualified, Type};
use crate::aster::pprint::Pretty;

use super::*;

pub(super) fn verify_typeof(lazy: &Lazy, ty: &(impl TypeOf + GetSpan + Pretty<Out = String>)) -> Result<()> {
  let Some(ty) = ty.type_of(lazy)? else {
    return Err(Box::new(Error::UnresolvedInVerify {
      what: ty.print(lazy),
      span: ty.get_span(lazy),
    }));
  };

  verify_type(lazy, &ty)
}

pub(super) fn verify_type(lazy: &Lazy, ty: &Type) -> Result<()> {
  match ty {
    Type::Reference(type_reference) => verify_typeof(lazy, type_reference),

    | Type::Resolved { part: ty, .. }
    | Type::ReferenceTo { ty, .. }
    | Type::UnsizedArrayOf { ty, .. }
    | Type::SizedArrayOf { ty, .. } => verify_type(lazy, ty.rget_from(lazy)),

    | &Type::Unresolved { qualified: Qualified { span, .. }, .. }
    | &Type::WeakInteger { span, .. }
    | &Type::WeakFloat { span, .. }
    | &Type::WeakString { span, .. }
    | &Type::Weak { span, .. }
      => Err(Box::new(Error::UnresolvedInVerify {
        what: ty.print(lazy),
        span,
      })),
    Type::Intrinsic { kind, span } => Ok(()),
  }
}
