use crate::lang::Lazy;
use crate::resolve::reference::{BlockReference, ExpressionReference, Reference};
use crate::tokenize::token::{NumericValue, Span};
use crate::lang::ty::Type;
use crate::lang::function::{BlockId, ExprId};

#[derive(Debug)]
pub struct BlockExpression {
  pub children: Vec<ExprId>,
  pub span: Span,
  pub returns_last: bool,
  pub out: Type,
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
  pub fn new(span: Span, out: Type) -> Self {
    Self {
      children: vec![],
      span,
      returns_last: false,
      out,
    }
  }
}

impl BlockReference {
  pub fn get_return_last(&self, lazy: &Lazy) -> Option<ExpressionReference> {
    let block = self.rget_from(lazy);

    if !block.returns_last {
      return None;
    };

    let &index = block.children.last().unwrap();

    Some(ExpressionReference { function: self.function, index })
  }
}
