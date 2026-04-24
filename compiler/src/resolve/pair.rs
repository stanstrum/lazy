use crate::lang::ty::Type;
use crate::lang::reference::TypeReference;
use crate::resolve::tasks::OverwriteTypeReference;

#[derive(Debug, Clone, Copy)]
pub enum TypePairModifier {
  Dereference,
}

#[derive(Debug, Clone)]
pub struct TypePair {
  pub overwrite: OverwriteTypeReference,
  pub ty: Type,
}

impl TypePair {
  pub fn new(reference: TypeReference, ty: Type) -> Self {
    Self {
      overwrite: reference.into(),
      ty,
    }
  }
}
