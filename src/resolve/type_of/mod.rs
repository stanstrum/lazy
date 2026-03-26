use crate::lang::Lazy;
use crate::lang::expr::{BlockExpression, Expression, LiteralKind, Variable};
use crate::lang::ty::Type;
use crate::lang::reference::Store;
use crate::tokenize::token::NumericValue;

use super::*;
pub trait TypeOf {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>>;
}

impl<R: Copy> TypeOf for R
  where for<'a> Lazy<'a>: Store<R>,
        for<'a> <Lazy<'a> as Store<R>>::Out: TypeOf
{
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    lazy.rget(*self).type_of(lazy)
  }
}

impl TypeOf for Type {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    match self {
      | Type::Intrinsic { .. }
      | Type::WeakInteger { .. }
      | Type::WeakFloat { .. }
      | Type::WeakString { .. }
      | Type::Weak { .. }
      | Type::ReferenceTo { .. }
      | Type::UnsizedArrayOf { .. }
      | Type::SizedArrayOf { .. }
      | Type::Unresolved { .. }
      => Ok(Some(self.clone())),
      // SPONGE
      // | Type::Unresolved { .. }
      //   => Ok(None),
      Type::Resolved { part, .. } => part.type_of(lazy),
      Type::Reference(reference) => reference.type_of(lazy),
    }
  }
}

impl TypeOf for BlockExpression {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    // TODO: is this correct? should I try to match the expr type directly,
    //       maybe in addition to this?  Coerce in TypeOf? what could go
    //       wrong ???
    self.out.type_of(lazy)
  }
}

impl TypeOf for Variable {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    self.ty.type_of(lazy)
  }
}

impl TypeOf for Expression {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    match self {
      Expression::Block(block) => block.type_of(lazy),
      Expression::Variable { reference, .. } => reference.type_of(lazy),
      // TODO: again, very unsure about this... we are relying on the Resolve
      //       mechanism to hit the insides of the Expression and then
      //       looping to finish the job.  is this Functional™?
      | Expression::Literal { out, .. }
      | Expression::Unknown { out, .. }
      | Expression::Unary { out, .. }
      | Expression::Binary { out, .. }
        => out.type_of(lazy)
    }
  }
}
