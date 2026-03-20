use crate::lang::Lazy;
use crate::lang::expr::{Expression, LiteralKind};
use crate::lang::ty::Type;
use crate::lang::reference::{ExpressionReference, Reference, Store, TypeReference};
use crate::tokenize::token::NumericValue;

use super::*;
pub trait TypeOf {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>>;
}

// impl TypeOf for TypeReference {
//   fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
//     match self {
//       TypeReference::Part(part) => {
//         part.rget_from(lazy).type_of(lazy)
//       },
//       TypeReference::ReturnTypeOf(function) => {
//         (&function.rget_from(lazy).header.ret_ty).type_of(lazy)
//       },
//       TypeReference::Alias(_) => todo!(),
//       TypeReference::ArgumentOf(..) => todo!(),
//     }
//   }
// }

impl<R: Copy> TypeOf for R
  where for<'a> Lazy<'a>: Store<R>,
        for<'a> <Lazy<'a> as Store<R>>::Out: TypeOf
{
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    lazy.rget(*self).type_of(lazy)
  }
}

impl<'a> TypeOf for ResolvedTypePair<'a> {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    // should i be cloning this?
    Ok(Some(self.1.clone()))
  }
}

impl TypeOf for Type {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    match self {
      | Type::Intrinsic { .. }
      | Type::Unresolved { .. }
      | Type::WeakInteger { .. }
      | Type::WeakFloat { .. }
      | Type::WeakString { .. }
      | Type::ReferenceTo { .. }
      | Type::UnsizedArrayOf { .. }
      | Type::SizedArrayOf { .. } => Ok(Some(self.clone())),
      // Type::Expression(expression) => expression.rget_from(lazy).type_of(lazy),
      Type::Reference(reference) => reference.type_of(lazy),
    }
  }
}

impl TypeOf for &Expression {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    match self {
      Expression::Block(block_reference) => todo!(),
      Expression::Literal { value, span, out } => {
        Ok(Some(match value {
          LiteralKind::Numeric(NumericValue::U64(_)) => Type::WeakFloat { span: *span },
          LiteralKind::Numeric(NumericValue::F64(_)) => Type::WeakInteger { span: *span },
          LiteralKind::String { value, kind } => Type::WeakString { span: *span },
        }))
      },
      Expression::Variable { reference, span } => todo!(),
      Expression::Unknown(qualified) => todo!(),
      Expression::Unary { expr, op, span } => todo!(),
      Expression::Binary { a, b, op, span } => todo!(),
    }
  }
}

impl TypeOf for ExpressionReference {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    todo!()
  }
}
