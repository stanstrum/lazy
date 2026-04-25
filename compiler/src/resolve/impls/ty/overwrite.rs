use super::*;

impl Coerce for OverwriteTypeReference {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let Some(ty) = self.type_of(lazy) else {
      return Ok(());
    };

    TypePair {
      overwrite: self.clone(),
      ty,
    }.coerce(lazy, other, tasks)
  }
}

impl<'a> Store<OverwriteTypeReference> for Lazy<'a> {
  type Out = Type;

  fn rget(&self, _key: OverwriteTypeReference) -> &Self::Out {
    todo!()
  }

  fn rget_mut(&mut self, key: OverwriteTypeReference) -> &mut Self::Out {
    let ty = self.rget_mut(key.reference);

    for modifier in key.modifiers.iter() {
      match modifier {
        TypePairModifier::Dereference => todo!(),
      };
    };

    ty
  }
}
