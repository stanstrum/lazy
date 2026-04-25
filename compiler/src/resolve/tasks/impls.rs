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
