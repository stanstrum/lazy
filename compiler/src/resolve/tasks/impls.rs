use crate::aster::pprint::Pretty;
use crate::lang::expr::Expression;
use crate::lang::{ExpressionReference, TypeReference};

use super::*;

pub struct Subjugate {
  pub prerequisite: Box<dyn Task>,
  pub after: Box<dyn Task>,
}

pub struct OverwriteExpression {
  pub dest: ExpressionReference,
  pub src: Expression,
}

pub type OverwriteTypeReference = ::lang::ty::OverwriteTypeReference<crate::LazyStructures>;
pub struct OverwriteType {
  pub dest: OverwriteTypeReference,
  pub src: Type,
}

pub struct ResolveAsTask<R: Resolve> {
  pub reference: R,
}

impl From<TypePair> for OverwriteTypeReference {
  fn from(value: TypePair) -> Self {
    value.overwrite.clone()
  }
}

impl Pretty for OverwriteTypeReference {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    let ty = self.reference.rget_from(lazy);

    TypePair {
      overwrite: self.reference.into(),
      ty: ty.clone(),
    }.print(lazy)
  }
}

impl From<TypeReference> for OverwriteTypeReference {
  fn from(dest: TypeReference) -> Self {
    Self {
      reference: dest,
      modifiers: vec![],
    }
  }
}

// TODO: these should go to `lang`
impl<C: Compiler> GetSpan<C> for TypePair<C> {
  fn get_span(&self, store: &C::Store<'_>) -> Span<C> {
    self.overwrite.get_span(store)
  }
}

impl<C: Compiler> GetSpan<C> for OverwriteTypeReference<C> {
  fn get_span(&self, store: &C::Store<'_>) -> Span<C> {
    self.reference.get_span(store)
  }
}
