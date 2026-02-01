use crate::aster::pprint::Pretty;
use crate::lang::module::FunctionId;
use crate::lang::function::ExprId;

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

pub(super) fn assert_assignable_expr(lazy: &Lazy, function: FunctionId, expr: ExprId, ty: &lang::ty::Type) -> Result<(), Box<Error>> {
  match &lazy[function][expr] {
    lang::expr::Expression::BlockExpression(block_id) => todo!(),
    lang::expr::Expression::Literal { value, span, out } => todo!(),
  }
}

pub(super) fn coerce(lazy: &Lazy, what: &TypeReference, to: lang::ty::Type) -> Result<(), Box<Error>> {
  todo!()
}
