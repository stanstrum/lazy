use crate::Compiler;
use crate::reference::{ExpressionReference, TypeReference};
use crate::expr::Expression;
use crate::ty::{OverwriteTypeReference, Type, TypeOf, TypePair};

impl<C: Compiler> TypeOf<C> for Type<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type> {
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
      Type::Resolved { part, .. } => part.type_of(lazy),
      Type::Reference(reference) => reference.type_of(lazy),
    }
  }

  fn reference(&self, _store: &C::Store<'_>) -> Option<OverwriteTypeReference> {
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

impl<C: Compiler> TypeOf<C> for TypeReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type> {
    self.rget_from(lazy).type_of(lazy)
  }

  fn reference(&self, _store: &C::Store<'_>) -> Option<OverwriteTypeReference> {
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

  fn reference(&self, store: &C::Store<'_>) -> Option<OverwriteTypeReference<C>> {
    Some(TypeReference::Expression(*self).into())
  }
}

impl<C: Compiler> TypeOf for TypePair<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type> {
    self.ty.type_of(lazy)
  }

  fn reference(&self, _store: &C::Store<'_>) -> Option<OverwriteTypeReference> {
    Some(self.clone().into())
  }
}

impl<C: Compiler> TypeOf<C> for OverwriteTypeReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type> {
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

  fn reference(&self, _store: &C::Store<'_>) -> Option<OverwriteTypeReference> {
    todo!()
  }
}

impl<C: Compiler> TypeOf<C> for VariableReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type> {
    TypeReference::Variable(*self).type_of(lazy)
  }

  fn reference(&self, _store: &C::Store<'_>) -> Option<OverwriteTypeReference> {
    Some(TypeReference::Variable(*self).into())
  }
}

impl<C: Compiler> TypeOf<C> for BlockReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type> {
    // TODO: is this correct? should I try to match the expr type directly,
    //       maybe in addition to this?  Coerce in TypeOf? what could go
    //       wrong ???
    let reference = TypeReference::Block(*self);
    let ty = &self.rget_from(lazy).out;

    OverwriteTypeReference::from(TypePair::new(reference, ty.clone())).type_of(lazy)
  }

  fn reference(&self, _store: &C::Store<'_>) -> Option<OverwriteTypeReference> {
    Some(TypeReference::Block(*self).into())
  }
}
