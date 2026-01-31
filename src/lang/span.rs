use crate::lang::Lazy;
use crate::resolve::reference::Reference;
use crate::{tokenize::token::Span};
use crate::lang::ty::{Intrinsic, Type};
use crate::lang::module::Module;
use crate::lang::function::Function;
use crate::lang::expr::{BlockExpression, Expression};

pub trait GetSpan {
  type Parent<'a>;

  fn get_span(&self, parent: &Self::Parent<'_>) -> Span;
}

impl GetSpan for Module {
  type Parent<'a> = Lazy<'a>;

  fn get_span(&self, _lazy: &Lazy) -> Span {
    todo!()
  }
}

impl GetSpan for Function {
  type Parent<'a> = Lazy<'a>;

  fn get_span(&self, _lazy: &Lazy) -> Span {
    todo!()
  }
}

impl GetSpan for Type {
  type Parent<'a> = Lazy<'a>;

  fn get_span(&self, parent: &Lazy) -> Span {
    match self {
      Type::Unresolved { qualified, .. } => qualified.span,
      Type::Resolved { original, .. } => original.get_span(parent),
      | Type::ReferenceTo { span, .. }
      | Type::SizedArrayOf { span, .. }
      | Type::UnsizedArrayOf { span, .. }
      | Type::Intrinsic { span, .. }
      | Type::WeakInteger { span }
      | Type::WeakFloat { span } => *span,
      Type::Reference(reference) => reference.rget_from(parent).get_span(parent),
    }
  }
}

impl GetSpan for Intrinsic {
  type Parent<'a> = Lazy<'a>;

  fn get_span(&self, _lazy: &Lazy) -> Span {
    todo!()
  }
}

impl GetSpan for Expression {
  type Parent<'a> = Function;

  fn get_span(&self, parent: &Function) -> Span {
    match self {
      Expression::BlockExpression(id) => parent[*id].get_span(&()),
      Expression::Literal { span, .. } => *span,
    }
  }
}

impl GetSpan for BlockExpression {
  type Parent<'a> = ();

  fn get_span(&self, _parent: &()) -> Span {
    self.span
  }
}
