use crate::lang::Lazy;
use crate::lang::reference::{Reference, Store};
use crate::tokenize::token::Span;
use crate::lang::ty::Type;
use crate::lang::module::{Module, TypeAlias};
use crate::lang::function::Function;
use crate::lang::expr::{BlockExpression, Expression};

pub trait GetSpan {
  fn get_span(&self, lazy: &Lazy) -> Span;
}

impl<R: for<'a> Reference<Lazy<'a>>> GetSpan for R
  where for<'a> Lazy<'a>: Store<R>,
        for<'a> <Lazy<'a> as Store<R>>::Out: GetSpan
{
  fn get_span(&self, lazy: &Lazy) -> Span {
    self.rget_from(lazy).get_span(lazy)
  }
}

impl GetSpan for Module {
  fn get_span(&self, _lazy: &Lazy) -> Span {
    todo!()
  }
}

impl GetSpan for Function {
  fn get_span(&self, _lazy: &Lazy) -> Span {
    self.span
  }
}

impl GetSpan for TypeAlias {
  fn get_span(&self, _lazy: &Lazy) -> Span {
    self.span
  }
}

impl GetSpan for Type {
  fn get_span(&self, lazy: &Lazy) -> Span {
    match self {
      Type::Unresolved { qualified, .. } => qualified.span,
      | Type::Resolved { span, .. }
      | Type::ReferenceTo { span, .. }
      | Type::SizedArrayOf { span, .. }
      | Type::UnsizedArrayOf { span, .. }
      | Type::Intrinsic { span, .. }
      | Type::Weak { span }
      | Type::WeakInteger { span }
      | Type::WeakFloat { span }
      | Type::WeakString { span, .. }
        => *span,
      Type::Reference(reference) => reference.get_span(lazy),
    }
  }
}

impl GetSpan for Expression {
  fn get_span(&self, lazy: &Lazy) -> Span {
    match self {
      Expression::Block(id) => id.rget_from(lazy).get_span(lazy),
      | Expression::Literal { span, .. }
      | Expression::Variable { span, .. }
      | Expression::Binary { span, .. }
      | Expression::Unary { span, .. }
        => *span,
      Expression::Unknown { qualified, .. } => qualified.span,
    }
  }
}

impl GetSpan for BlockExpression {
  fn get_span(&self, _lazy: &Lazy) -> Span {
    self.span
  }
}
