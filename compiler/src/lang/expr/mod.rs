pub mod operator;

use crate::lang::expr::operator::{BinaryOperator, UnaryOperator};
use crate::lang::module::Name;
use crate::lang::reference::{BlockReference, ExpressionReference, VariableReference};
use crate::lang::span::GetSpan;
use string_pool::StringId;
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
  pub parent: Option<BlockReference>,
  pub children: Vec<ExprId>,
  pub span: Span,
  pub returns_last: bool,
  pub out: Type,
  pub variables: Vec<Variable>,
}

#[derive(Debug, Clone, Copy)]
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
  Unknown {
    qualified: Qualified,
    out: Type,
  },
  Unary {
    expr: ExpressionReference,
    op: (UnaryOperator, Span),
    span: Span,
    out: Type,
  },
  Binary {
    a: ExpressionReference,
    b: ExpressionReference,
    op: (BinaryOperator, Span),
    span: Span,
    out: Type,
  },
  StructInitializer {
    ty: Type,
    members: Vec<(Name, ExpressionReference)>,
    span: Span,
  },
}

impl Expression {
  pub fn new_unknown(qualified: Qualified) -> Self {
    let span = qualified.span;

    Self::Unknown {
      qualified,
      out: Type::Weak { span },
    }
  }
}

impl BlockExpression {
  pub fn new_dirty(parent: Option<BlockReference>, temp_span: Span) -> Self {
    Self::new(
      parent,
      temp_span,
      Type::Intrinsic {
        kind: ::token::intrinsic::Intrinsic::Void,
        span: temp_span,
      },
    )
  }

  pub fn new(parent: Option<BlockReference>, span: Span, out: Type) -> Self {
    Self {
      parent,
      children: vec![],
      span,
      returns_last: false,
      out,
      variables: vec![],
    }
  }
}

impl GetSpan for BlockExpression {
  fn get_span(&self, _lazy: &crate::Lazy) -> Span {
    self.span
  }
}
