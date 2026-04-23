use crate::lang::ty::Type;
use crate::lang::reference::TypeReference;

#[derive(Debug, Clone, Copy)]
pub enum TypePairModifier {
  Dereference,
}

#[derive(Debug, Clone)]
pub struct TypePair {
  pub reference: TypeReference,
  pub ty: Type,
  pub modifiers: Vec<TypePairModifier>,
}

impl TypePair {
  pub fn new(reference: TypeReference, ty: Type) -> Self {
    Self {
      reference,
      ty,
      modifiers: vec![],
    }
  }
}
