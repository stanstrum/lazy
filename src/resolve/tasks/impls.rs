use crate::lang::expr::Expression;
use crate::lang::reference::{ExpressionReference, TypeReference};

use super::*;

pub struct Subjugate {
  pub prerequisite: Box<dyn Task>,
  pub after: Box<dyn Task>,
}

pub struct OverwriteExpression {
  pub dest: ExpressionReference,
  pub src: Expression,
}

pub struct OverwriteType {
  pub dest: TypeReference,
  pub src: Type,
}

pub struct ResolveAsTask<R: Resolve> {
  pub reference: R,
}

