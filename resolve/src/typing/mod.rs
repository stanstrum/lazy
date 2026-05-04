pub(crate) mod module;
pub(crate) mod function;
pub(crate) mod expr;

use lang::reference::{BlockReference, ExpressionReference, Store, TypeReference, VariableReference};

use super::*;

pub(crate) trait Resolve<C: Compiler> {
  /// Resolves with the given context [`Resolver`].  Returns `true` if the
  /// object in question should be considered "complete", or `false`
  fn resolve(&self, resolver: &Resolver<C>) -> Result<C, bool>;
}

pub(crate) trait Coerce<C: Compiler> {
  fn coerce(&self, resolver: &Resolver<C>, other: &impl TypeOf<C>) -> Result<C>;
}

pub(crate) trait Typify<C: Compiler>: Sized {
  fn get_type_iter<'store>(self, store: &'store C::Store<'_>) -> Box<dyn Iterator<Item = TypeReference<C>> + 'store>;
}

impl<C: Compiler> Coerce<C> for Type<C> {
  fn coerce(&self, resolver: &Resolver<C>, other: &impl TypeOf<C>) -> Result<C> {
    todo!()
  }
}
