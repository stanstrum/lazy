use lang::Compiler;
use ::pprint::Pretty;
use ::lang::expr::Expression;
use ::gluezy::{TypeReference};
use ::lang::reference::ExpressionReference;

use super::*;

pub struct Subjugate<C: Compiler> {
  pub prerequisite: Box<dyn Task<C>>,
  pub after: Box<dyn Task<C>>,
}

pub struct OverwriteExpression<C: Compiler> {
  pub dest: ExpressionReference<C>,
  pub src: Expression<C>,
}

pub type OverwriteTypeReference = ::lang::ty::OverwriteTypeReference<gluezy::LazyStructures>;
pub struct OverwriteType {
  pub dest: OverwriteTypeReference,
  pub src: Type,
}

pub struct ResolveAsTask<R: Resolve> {
  pub reference: R,
}
