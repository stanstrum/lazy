use crate::Compiler;
use crate::reference::{BlockReference, ExpressionReference, Reference, TypePartReference, TypeReference, VariableReference};
use crate::expr::Expression;
use crate::ty::{TypeValue, TypeOf, Type};

impl<C: Compiler> TypeOf<C> for TypeValue<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>> {
    match self {
      | TypeValue::Intrinsic { .. }
      | TypeValue::WeakInteger { .. }
      | TypeValue::WeakFloat { .. }
      | TypeValue::WeakString { .. }
      | TypeValue::Weak { .. }
      | TypeValue::ReferenceTo { .. }
      | TypeValue::UnsizedArrayOf { .. }
      | TypeValue::SizedArrayOf { .. }
      // | Type::Unresolved { .. }
      | TypeValue::Struct { .. }
        => todo!(),
        // => Some(self.clone()),
      // SPONGE
      | TypeValue::Unresolved { .. }
        => None,
      TypeValue::Resolved { part, .. } => part.type_of(store),
      TypeValue::Reference(reference) => reference.type_of(store),
    }
  }

}

impl<C: Compiler> TypeOf<C> for TypePartReference<C> {
  fn type_of(&self, store: &<C as Compiler>::Store<'_>) -> Option<Type<C>> {
    self.rget_from(store).type_of(store)
  }
}

impl<C: Compiler> TypeOf<C> for TypeReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>> {
    self.rget_from(store).type_of(store)
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

          todo!()
          // Type::new(reference, out.clone()).type_of(store)
        },
    }
  }
}

impl<C: Compiler> TypeOf<C> for Type<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>> {
    todo!()
    // self.ty.type_of(store)
  }
}

impl<C: Compiler> TypeOf<C> for VariableReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>> {
    TypeReference::Variable(*self).type_of(store)
  }
}

impl<C: Compiler> TypeOf<C> for BlockReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>> {
    // TODO: is this correct? should I try to match the expr type directly,
    //       maybe in addition to this?  Coerce in TypeOf? what could go
    //       wrong ???
    let reference = TypeReference::Block(*self);
    let ty = &self.rget_from(store).out;

    todo!()
    // OverwriteTypeReference::from(TypePair::new(reference, ty.clone())).type_of(store)
  }
}
