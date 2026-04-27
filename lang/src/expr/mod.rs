pub mod operator;

use string_pool::StringId;

use crate::Compiler;
use crate::reference::{BlockReference, ExpressionReference, VariableReference};
use crate::ty::{Qualified, TypeKind};
use crate::token::{NumericValue, StringKind};
use crate::span::Span;
use crate::module::Name;
use crate::function::ExprId;

#[derive(Debug)]
pub struct Variable<C: Compiler> {
  pub name: Name<C>,
  pub ty: TypeKind<C>,
  pub span: Span<C>,
}

#[derive(Debug)]
pub struct BlockExpression<C: Compiler> {
  pub parent: Option<BlockReference<C>>,
  pub children: Vec<ExprId>,
  pub span: Span<C>,
  pub returns_last: bool,
  pub out: TypeKind<C>,
  pub variables: Vec<Variable<C>>,
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
pub enum Expression<C: Compiler> {
  Block(BlockReference<C>),
  Literal {
    value: LiteralKind,
    span: Span<C>,
    out: TypeKind<C>,
  },
  Variable {
    reference: VariableReference<C>,
    span: Span<C>,
  },
  Unknown {
    qualified: Qualified<C>,
    out: TypeKind<C>,
  },
  Unary {
    expr: ExpressionReference<C>,
    op: (operator::UnaryOperator<C>, Span<C>),
    span: Span<C>,
    out: TypeKind<C>,
  },
  Binary {
    a: ExpressionReference<C>,
    b: ExpressionReference<C>,
    op: (operator::BinaryOperator, Span<C>),
    span: Span<C>,
    out: TypeKind<C>,
  },
  StructInitializer {
    ty: TypeKind<C>,
    members: Vec<(Name<C>, ExpressionReference<C>)>,
    span: Span<C>,
  },
}

impl<C: Compiler> Expression<C> {
  pub fn new_unknown(qualified: Qualified<C>) -> Self {
    let span = qualified.span;

    Self::Unknown {
      qualified,
      out: TypeKind::Weak { span },
    }
  }
}

impl<C: Compiler> BlockExpression<C> {
  pub fn new_dirty(parent: Option<BlockReference<C>>, temp_span: Span<C>) -> Self {
    Self::new(
      parent,
      temp_span,
      TypeKind::Intrinsic {
        kind: crate::intrinsic::Intrinsic::Void,
        span: temp_span,
      },
    )
  }

  pub fn new(parent: Option<BlockReference<C>>, span: Span<C>, out: TypeKind<C>) -> Self {
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
