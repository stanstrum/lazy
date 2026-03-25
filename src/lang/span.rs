use crate::lang::Lazy;
use crate::lang::reference::{Reference, TypePartReference, TypeReference};
use crate::tokenize::token::Span;
use crate::lang::ty::Type;
use crate::lang::module::{Module, TypeAlias};
use crate::lang::function::Function;
use crate::lang::expr::{BlockExpression, Expression};

pub trait GetSpan {
  fn get_span(&self, lazy: &Lazy) -> Span;
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

impl GetSpan for TypeReference {
  fn get_span(&self, lazy: &Lazy) -> Span {
    match self {
      TypeReference::Part(type_part) => type_part.rget_from(lazy).get_span(lazy),
      TypeReference::ReturnTypeOf(function) => function.rget_from(lazy).header.ret_ty.get_span(lazy),
      TypeReference::Alias(alias) => alias.rget_from(lazy).span,
      TypeReference::Variable(variable) => variable.rget_from(lazy).span,
      TypeReference::Expression(_) => todo!(),
      TypeReference::Block(_) => todo!(),
    }
  }
}

impl GetSpan for TypePartReference {
  fn get_span(&self, lazy: &Lazy) -> Span {
    self.rget_from(lazy).get_span(lazy)
  }
}

impl GetSpan for Type {
  fn get_span(&self, lazy: &Lazy) -> Span {
    match self {
      Type::Unresolved { qualified, .. } => qualified.span,
      Type::Resolved { part, .. } => part.get_span(lazy),
      | Type::ReferenceTo { span, .. }
      | Type::SizedArrayOf { span, .. }
      | Type::UnsizedArrayOf { span, .. }
      | Type::Intrinsic { span, .. }
      | Type::WeakInteger { span }
      | Type::WeakFloat { span } => *span,
      Type::Reference(reference) => reference.get_span(lazy),
      // Type::Expression(reference) => {
      //   let function = &parent[reference.function];
      //   reference.rget_from(parent).get_span(function)
      // },
      other => todo!("{other:?}"),
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
