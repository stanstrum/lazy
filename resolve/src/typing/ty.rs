use super::*;

impl<C: Compiler> Resolve<C> for TypeReference<C> {
  fn resolve(&self, resolver: &Resolver<C>) -> Result<C, bool> {
    Into::<Type<C>>::into(*self).resolve(resolver)
  }
}

impl<C: Compiler> Resolve<C> for ResolvedType<C> {
  fn resolve(&self, _resolver: &Resolver<C>) -> Result<C, bool> {
    match &self.ty {
      TypeValue::Intrinsic { .. } => Ok(true),
      | TypeValue::Unresolved { .. }
      | TypeValue::WeakInteger { .. }
      | TypeValue::WeakFloat { .. }
      | TypeValue::WeakString { .. }
      | TypeValue::Weak { .. }
        => Ok(false),
      other => todo!("{other:#?}"),
    }
  }
}

impl<C: Compiler> Resolve<C> for Type<C> {
  fn resolve(&self, resolver: &Resolver<C>) -> Result<C, bool> {
    let Some(ty) = self.type_of(resolver.store) else {
      return Ok(false);
    };

    ty.resolve(resolver)
  }
}
