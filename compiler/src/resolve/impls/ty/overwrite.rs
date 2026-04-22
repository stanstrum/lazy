use super::*;

impl TypeOf for OverwriteTypeReference {
  fn type_of(&self, lazy: &Lazy) -> Option<Type> {
    let ty = self.reference.rget_from(lazy).to_owned();

    for modifier in self.modifiers.iter() {
      match modifier {
        TypePairModifier::Dereference => match ty {
          Type::Reference(_) => todo!(),
          Type::Resolved { .. } => todo!(),
          Type::Unresolved { .. } => todo!(),
          Type::Intrinsic { .. } => todo!(),
          Type::WeakInteger { .. } => todo!(),
          Type::WeakFloat { .. } => todo!(),
          Type::WeakString { .. } => todo!(),
          Type::Weak { .. } => todo!(),
          Type::ReferenceTo { .. } => todo!(),
          Type::UnsizedArrayOf { .. } => todo!(),
          Type::SizedArrayOf { .. } => todo!(),
          Type::Struct { .. } => todo!(),
        },
      };
    };

    Some(ty)
  }

  fn reference(&self, _lazy: &Lazy) -> Option<OverwriteTypeReference> {
    todo!()
  }
}

impl Coerce for OverwriteTypeReference {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let Some(ty) = self.type_of(lazy) else {
      return Ok(());
    };

    TypePair {
      reference: self.reference,
      modifiers: self.modifiers.clone(),
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
