pub mod operator;

use crate::lang::expr::operator::{BinaryOperator, UnaryOperator};
use crate::lang::module::Name;
use crate::lang::reference::{BlockReference, ExpressionReference, VariableReference};
use crate::string_pool::StringId;
use crate::tokenize::token::{NumericValue, Span, StringKind};
use crate::lang::ty::{Qualified, Type};
use crate::lang::function::ExprId;

#[derive(Debug)]
pub struct Variable {
  pub name: Name,
  pub ty: Type,
  pub span: Span,
}

#[derive(Debug)]
pub struct BlockExpression {
  pub children: Vec<ExprId>,
  pub span: Span,
  pub returns_last: bool,
  pub out: Type,
  pub variables: Vec<Variable>,
}

#[derive(Debug)]
pub enum LiteralKind {
  Numeric(NumericValue),
  String {
    value: StringId,
    kind: StringKind,
  },
}

#[derive(Debug)]
pub enum Expression {
  Block(BlockReference),
  Literal {
    value: LiteralKind,
    span: Span,
    out: Type,
  },
  Variable {
    reference: VariableReference,
    span: Span,
  },
  Unknown(Qualified),
  Unary {
    expr: ExpressionReference,
    op: (UnaryOperator, Span),
    span: Span,
  },
  Binary {
    a: ExpressionReference,
    b: ExpressionReference,
    op: (BinaryOperator, Span),
    span: Span,
  },
}

impl BlockExpression {
  pub fn new_dirty(temp_span: Span) -> Self {
    Self::new(temp_span, Type::Intrinsic {
        kind: crate::lang::ty::Intrinsic::Void,
        span: temp_span,
      },
    )
  }

  pub fn new(span: Span, out: Type) -> Self {
    Self {
      children: vec![],
      span,
      returns_last: false,
      out,
      variables: vec![],
    }
  }
}

// impl BlockReference {
//   pub fn get_return_last(&self, lazy: &Lazy) -> Option<ExpressionReference> {
//     let block = self.rget_from(lazy);

//     if !block.returns_last {
//       return None;
//     };

//     let &index = block.children.last().unwrap();

//     Some(ExpressionReference { function: self.function, index })
//   }
// }
