use crate::tokenize::token::{NumericValue, Span};
use crate::lang::function::BlockId;

#[derive(Debug)]
pub struct BlockExpression {
  pub children: Vec<Expression>,
  pub span: Span,
}

#[derive(Debug)]
pub enum Expression {
  BlockExpression(BlockId),
  Literal {
    value: NumericValue,
    span: Span,
  },
}

impl BlockExpression {
  pub fn new(span: Span) -> Self {
    Self {
      children: vec![],
      span,
    }
  }
}
