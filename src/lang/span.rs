use crate::lang::Lazy;
use crate::lang::reference::{Reference, TypePartReference, TypeReference};
use crate::tokenize::token::Span;
use crate::lang::ty::Type;
use crate::lang::module::{Module, TypeAlias};
use crate::lang::function::Function;
use crate::lang::expr::{BlockExpression, Expression};

pub trait GetSpan {
  type Parent<'a>;

  fn get_span(&self, parent: Self::Parent<'_>) -> Span;
}

impl GetSpan for Module {
  type Parent<'a> = ();

  fn get_span(&self, _parent: ()) -> Span {
    todo!()
  }
}

impl GetSpan for Function {
  type Parent<'a> = ();

  fn get_span(&self, _parent: ()) -> Span {
    self.span
  }
}

impl GetSpan for TypeAlias {
  type Parent<'a> = ();

  fn get_span(&self, _parent: ()) -> Span {
    self.span
  }
}

impl GetSpan for TypeReference {
  type Parent<'a> = &'a Lazy<'a>;

  fn get_span(&self, parent: Self::Parent<'_>) -> Span {
    match self {
      TypeReference::Part(type_part) => type_part.rget_from(parent).get_span(parent),
      TypeReference::ReturnTypeOf(function) => function.rget_from(parent).header.ret_ty.get_span(parent),
    }
  }
}

impl GetSpan for TypePartReference {
  type Parent<'a> = &'a Lazy<'a>;

  fn get_span(&self, parent: Self::Parent<'_>) -> Span {
    self.rget_from(parent).get_span(parent)
  }
}

impl GetSpan for Type {
  type Parent<'a> = &'a Lazy<'a>;

  fn get_span(&self, _parent: &Lazy) -> Span {
    match self {
      Type::Unresolved { qualified, .. } => qualified.span,
      // Type::Resolved { original, .. } => original.get_span(parent),
      | Type::ReferenceTo { span, .. }
      | Type::SizedArrayOf { span, .. }
      | Type::UnsizedArrayOf { span, .. }
      | Type::Intrinsic { span, .. }
      | Type::WeakInteger { span }
      | Type::WeakFloat { span } => *span,
      // Type::Reference(reference) => reference.get_span(parent),
      // Type::Expression(reference) => {
      //   let function = &parent[reference.function];
      //   reference.rget_from(parent).get_span(function)
      // },
      other => todo!("{other:?}"),
    }
  }
}

impl GetSpan for Expression {
  type Parent<'a> = &'a Lazy<'a>;

  fn get_span(&self, lazy: &Lazy) -> Span {
    match self {
      Expression::Block(id) => id.rget_from(lazy).get_span(()),
      | Expression::Literal { span, .. }
      | Expression::Variable { span, .. }
      | Expression::Binary { span, .. }
      | Expression::Unary { span, .. }
        => *span,
      Expression::Unknown(qualified) => qualified.span,
    }
  }
}

impl GetSpan for BlockExpression {
  type Parent<'a> = ();

  fn get_span(&self, _parent: ()) -> Span {
    self.span
  }
}
