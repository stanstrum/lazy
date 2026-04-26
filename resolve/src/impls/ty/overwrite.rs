use super::*;

impl<C: Compiler + 'static> Coerce<C> for OverwriteTypeReference<C> {
  fn coerce(&self, store: &C::Store<'_>, other: &impl TypeOf<C>, tasks: &mut Tasks<C>) -> Result<C> {
    let Some(ty) = self.type_of(store) else {
      return Ok(());
    };

    TypePair {
      overwrite: self.clone(),
      ty,
    }.coerce(store, other, tasks)
  }
}
