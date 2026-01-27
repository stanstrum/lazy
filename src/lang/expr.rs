use crate::tokenize::token::{NumericValue, Span};
use crate::lang::ty::Type;
use crate::lang::function::{BlockId, ExprId};

#[derive(Debug)]
pub struct BlockExpression {
  pub children: Vec<ExprId>,
  pub span: Span,
  pub returns_last: bool,
}

#[derive(Debug)]
pub enum Expression {
  BlockExpression(BlockId),
  Literal {
    value: NumericValue,
    span: Span,
    out: Type,
  },
}

impl BlockExpression {
  pub fn new(span: Span) -> Self {
    Self {
      children: vec![],
      span,
      returns_last: false,
    }
  }
}
