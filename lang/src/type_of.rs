use crate::ty::{ResolvedType, Type, TypeOf, TypeValue};
use crate::reference::{BlockReference, ExpressionReference, Reference, Store, TypePartReference, TypeReference, VariableReference};
use crate::expr::Expression;
use crate::Compiler;

// impl<C: Compiler> TypeOf<C> for TypeValue<C> {
//   fn type_of(&self, store: &C::Store<'_>) -> Option<ResolvedType<C>> {
//     match self {
//       | TypeValue::Intrinsic { .. }
//       | TypeValue::WeakInteger { .. }
//       | TypeValue::WeakFloat { .. }
//       | TypeValue::WeakString { .. }
//       | TypeValue::Weak { .. }
//       | TypeValue::ReferenceTo { .. }
//       | TypeValue::UnsizedArrayOf { .. }
//       | TypeValue::SizedArrayOf { .. }
//       // | Type::Unresolved { .. }
//       | TypeValue::Struct { .. }
//         => todo!("{}", ),
//         // => Some(self.clone()),
//       // SPONGE
//       | TypeValue::Unresolved { .. }
//         => None,
//       TypeValue::Resolved { part, .. } => part.type_of(store),
//       TypeValue::Reference(reference) => reference.type_of(store),
//     }
//   }
// }

impl<C: Compiler> TypeOf<C> for TypePartReference<C> {
  fn type_of(&self, store: &<C as Compiler>::Store<'_>) -> Option<ResolvedType<C>> {
    self.rget_from(store).type_of(store)
  }
}

impl<C: Compiler> TypeOf<C> for TypeReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<ResolvedType<C>> {
    self.rget_from(store).type_of(store)
  }
}

impl<C: Compiler> TypeOf<C> for ExpressionReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<ResolvedType<C>> {
    match self.rget_from(store) {
      Expression::Block(block) => block.type_of(store),
      Expression::Variable { reference, .. } => reference.type_of(store),
      | Expression::Literal { out, .. }
      | Expression::Unary { out, .. }
      | Expression::Binary { out, .. }
      | Expression::StructInitializer { ty: out, .. }
        => {
          out.type_of(store)
        },
    }
  }
}

impl<C: Compiler> TypeOf<C> for Type<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<ResolvedType<C>> {
    let type_value = if let Some(ty) = self.ty.as_ref() {
      ty
    } else if let Some(ty) = self.reference.rget_from(store).ty.as_ref() {
      ty
    } else {
      return None;
    };

    let type_value = match type_value {
      TypeValue::Reference(type_reference) => todo!(),
      TypeValue::Resolved { part, span } => todo!(),
      TypeValue::Unresolved { module, qualified } => {
        return None;
      },
      TypeValue::Intrinsic { kind, span } => todo!(),
      | TypeValue::WeakInteger { .. }
      | TypeValue::WeakFloat { .. }
      | TypeValue::WeakString { .. }
      | TypeValue::Weak { .. }
        => type_value.clone(),
      TypeValue::ReferenceTo { ty, r#mut, span } => todo!(),
      TypeValue::UnsizedArrayOf { ty, span } => todo!(),
      TypeValue::SizedArrayOf { ty, size, span } => todo!(),
      TypeValue::Struct { prototype } => todo!(),
    };

    Some(ResolvedType::new(self.reference, type_value))
  }
}

impl<C: Compiler> TypeOf<C> for ResolvedType<C> {
  fn type_of(&self, store: &<C as Compiler>::Store<'_>) -> Option<ResolvedType<C>> {
    match &self.ty {
      TypeValue::Reference(type_reference) => type_reference.type_of(store),
      TypeValue::Resolved { part, .. } => part.type_of(store),
      TypeValue::Unresolved { module, qualified, .. } => todo!(),
      TypeValue::Intrinsic { kind, span } => todo!(),

      | TypeValue::WeakInteger { .. }
      | TypeValue::WeakFloat { .. }
      | TypeValue::WeakString { .. }
      | TypeValue::Weak { .. }
        => Some(Self::new(self.reference, self.ty.clone())),

      TypeValue::ReferenceTo { ty, r#mut, span } => todo!(),
      TypeValue::UnsizedArrayOf { ty, span } => todo!(),
      TypeValue::SizedArrayOf { ty, size, span } => todo!(),
      TypeValue::Struct { prototype } => todo!(),
    }
  }
}

impl<C: Compiler> TypeOf<C> for VariableReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<ResolvedType<C>> {
    TypeReference::Variable(*self).type_of(store)
  }
}

impl<C: Compiler> TypeOf<C> for BlockReference<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<ResolvedType<C>> {
    // TODO: is this correct? should I try to match the expr type directly,
    //       maybe in addition to this?  Coerce in TypeOf? what could go
    //       wrong ???
    let reference = TypeReference::Block(*self);
    let ty = &self.rget_from(store).out;

    todo!()
    // OverwriteTypeReference::from(TypePair::new(reference, ty.clone())).type_of(store)
  }
}
