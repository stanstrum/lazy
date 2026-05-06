use super::*;

impl<C: Compiler> Resolve<C> for TypeReference<C> {
  fn resolve(&self, resolver: &Resolver<C>) -> Result<C, bool> {
    Into::<Type<C>>::into(*self).resolve(resolver)
  }
}

impl<C: Compiler> Resolve<C> for ResolvedType<C> {
  fn resolve(&self, resolver: &Resolver<C>) -> Result<C, bool> {
    match &self.ty {
      TypeValue::Reference(type_reference) => todo!(),
      TypeValue::Resolved { part, span } => todo!(),
      TypeValue::Unresolved { module, qualified } => Ok(false),
      TypeValue::Intrinsic { kind, span } => Ok(true),
      TypeValue::WeakInteger { span } => Ok(false),
      TypeValue::WeakFloat { span } => Ok(false),
      TypeValue::WeakString { kind, characters, span, dereferenced } => Ok(false),
      TypeValue::Weak { span } => Ok(false),
      TypeValue::ReferenceTo { ty, r#mut, span } => todo!(),
      TypeValue::UnsizedArrayOf { ty, span } => todo!(),
      TypeValue::SizedArrayOf { ty, size, span } => todo!(),
      TypeValue::Struct { prototype } => todo!(),
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
