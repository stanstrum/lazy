use crate::aster::pprint::Pretty;
use crate::lang::expr::BlockExpression;
use crate::lang::ty::Type;
// use crate::lang::module::FunctionId;
// use crate::lang::function::ExprId;

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

// pub(super) fn assert_assignable_expr(lazy: &Lazy, function: FunctionId, expr: ExprId, ty: &lang::ty::Type) -> Result<(), Box<Error>> {
//   match &lazy[function][expr] {
//     lang::expr::Expression::BlockExpression(block_id) => todo!(),
//     lang::expr::Expression::Literal { value, span, out } => todo!(),
//   }
// }

pub(super) trait IsResolved {
  fn is_resolved(&self, lazy: &Lazy) -> Result<bool, Box<Error>>;
}

impl IsResolved for Type {
  fn is_resolved(&self, lazy: &Lazy) -> Result<bool, Box<Error>> {
    todo!()
  }
}

impl IsResolved for BlockExpression {
  fn is_resolved(&self, lazy: &Lazy) -> Result<bool, Box<Error>> {
    let Some(out) = &self.out else {
      return Ok(false);
    };

    out.is_resolved(lazy)
  }
}

pub(super) trait Coerce<R: for<'a> Reference<'a, Out = Self>>: std::fmt::Debug {
  fn coerce(&self, lazy: &Lazy, reference: &R, to: &TypeReference, tasks: &mut Tasks) -> Result<(), Box<Error>>;
}

impl Coerce<TypeReference> for Type {
  fn coerce(&self, lazy: &Lazy, reference: &TypeReference, to: &TypeReference, tasks: &mut Tasks) -> Result<(), Box<Error>> {
    todo!()
  }
}

impl Coerce<BlockReference> for BlockExpression {
  fn coerce(&self, lazy: &Lazy, reference: &BlockReference, to: &TypeReference, tasks: &mut Tasks) -> Result<(), Box<Error>> {
    self.out.coerce(lazy, &TypeReference::Block(reference.to_owned()), to, tasks)
  }
}
