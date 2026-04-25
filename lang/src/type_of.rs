use crate::Compiler;
use crate::reference::{BlockReference, ExpressionReference, Reference, TypePartReference, TypeReference, VariableReference};
use crate::expr::Expression;
use crate::ty::{OverwriteTypeReference, Type, TypeOf, TypePair, TypePairModifier};

impl<C: Compiler> TypeOf<C> for Type<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>> {
    match self {
      | Type::Intrinsic { .. }
      | Type::WeakInteger { .. }
      | Type::WeakFloat { .. }
      | Type::WeakString { .. }
      | Type::Weak { .. }
      | Type::ReferenceTo { .. }
      | Type::UnsizedArrayOf { .. }
      | Type::SizedArrayOf { .. }
      // | Type::Unresolved { .. }
      | Type::Struct { .. }
        => Some(self.clone()),
      // SPONGE
      | Type::Unresolved { .. }
        => None,
      Type::Resolved { part, .. } => part.type_of(store),
      Type::Reference(reference) => reference.type_of(store),
    }
  }

  fn reference(&self, _store: &C::Store<'_>) -> Option<OverwriteTypeReference<C>> {
    match dbg!(self) {
      &Type::Reference(type_reference) => Some(type_reference.into()),
      &Type::Resolved { part, .. } => Some(TypeReference::Part(part).into()),
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
    }
  }
}

impl<C: Compiler> TypeOf<C> for TypePartReference<C> {
  fn type_of(&self, store: &<C as Compiler>::Store<'_>) -> Option<Type<C>> {
    self.rget_from(store).type_of(store)
  }

  fn reference(&self, store: &<C as Compiler>::Store<'_>) -> Option<OverwriteTypeReference<C>> {
    self.rget_from(store).reference(store)
  }
}

impl<C: Compiler> TypeOf<C> for TypeReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>> {
    self.rget_from(store).type_of(store)
  }

  fn reference(&self, _store: &C::Store<'_>) -> Option<OverwriteTypeReference<C>> {
    Some((*self).into())
  }
}

impl<C: Compiler> TypeOf<C> for ExpressionReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>> {
    match self.rget_from(store) {
      Expression::Block(block) => block.type_of(store),
      Expression::Variable { reference, .. } => reference.type_of(store),
      // TODO: again, very unsure about this... we are relying on the Resolve
      //       mechanism to hit the insides of the Expression and then
      //       looping to finish the job.  is this Functional™?
      | Expression::Literal { out, .. }
      | Expression::Unknown { out, .. }
      | Expression::Unary { out, .. }
      | Expression::Binary { out, .. }
      | Expression::StructInitializer { ty: out, .. }
        => {
          let reference = TypeReference::Expression(*self);

          TypePair::new(reference, out.clone()).type_of(store)
        },
    }
  }

  fn reference(&self, _store: &C::Store<'_>) -> Option<OverwriteTypeReference<C>> {
    Some(TypeReference::Expression(*self).into())
  }
}

impl<C: Compiler> TypeOf<C> for TypePair<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>> {
    self.ty.type_of(store)
  }

  fn reference(&self, _store: &C::Store<'_>) -> Option<OverwriteTypeReference<C>> {
    Some(self.clone().into())
  }
}

impl<C: Compiler> TypeOf<C> for OverwriteTypeReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>> {
    let ty = self.reference.rget_from(store).to_owned();

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

  fn reference(&self, _store: &C::Store<'_>) -> Option<OverwriteTypeReference<C>> {
    todo!()
  }
}

impl<C: Compiler> TypeOf<C> for VariableReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>> {
    TypeReference::Variable(*self).type_of(store)
  }

  fn reference(&self, _store: &C::Store<'_>) -> Option<OverwriteTypeReference<C>> {
    Some(TypeReference::Variable(*self).into())
  }
}

impl<C: Compiler> TypeOf<C> for BlockReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>> {
    // TODO: is this correct? should I try to match the expr type directly,
    //       maybe in addition to this?  Coerce in TypeOf? what could go
    //       wrong ???
    let reference = TypeReference::Block(*self);
    let ty = &self.rget_from(store).out;

    OverwriteTypeReference::from(TypePair::new(reference, ty.clone())).type_of(store)
  }

  fn reference(&self, _store: &C::Store<'_>) -> Option<OverwriteTypeReference<C>> {
    Some(TypeReference::Block(*self).into())
  }
}
