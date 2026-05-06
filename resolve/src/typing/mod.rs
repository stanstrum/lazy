pub(crate) mod module;
pub(crate) mod function;
pub(crate) mod expr;
pub(crate) mod ty;

use lang::{reference::{BlockReference, ExpressionReference, Store, TypeReference, VariableReference}, ty::ResolvedType};
use lazy_macros::print_once_per_thread;

use super::*;

pub(crate) trait Resolve<C: Compiler> {
  /// Resolves with the given context [`Resolver`].  Returns `true` if the
  /// object in question should be considered "complete", or `false`
  fn resolve(&self, resolver: &Resolver<C>) -> Result<C, bool>;
}

pub(crate) trait Coerce<C: Compiler> {
  fn coerce(&self, resolver: &Resolver<C>, other: &ResolvedType<C>) -> Result<C>;
}

pub(crate) trait Typify<C: Compiler>: Sized {
  fn get_type_iter<'store>(self, store: &'store C::Store<'_>) -> Box<dyn Iterator<Item = Box<dyn Resolve<C>>> + 'store>;
}

impl<C: Compiler, T: TypeOf<C>> Coerce<C> for T {
  fn coerce(&self, resolver: &Resolver<C>, _other: &ResolvedType<C>) -> Result<C> {
    let Some(ty) = self.type_of(resolver.store) else {
      print_once_per_thread!(resolver.store, {
        level: Level::Stub,
        force: false,
        description: line_dbg!("There's no knowing if this is any good").into(),
        contents: MessageContents::None::<C>,
      });

      return Ok(());
    };

    todo!()
  }
}
