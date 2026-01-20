use crate::{tokenize::token::Span};
use crate::lang::ty::{Intrinsic, Type};
use crate::lang::module::Module;
use crate::lang::function::Function;
use crate::lang::expr::{BlockExpression, Expression};

pub trait GetSpan {
  fn get_span(&self) -> Span {
    todo!()
  }
}

impl GetSpan for Module {}
impl GetSpan for Function {}
impl GetSpan for Type {
  fn get_span(&self) -> Span {
    match self {
      Type::Unresolved { qualified, .. } => qualified.span,
      Type::Intrinsic { span, .. } => *span,
    }
  }
}

impl GetSpan for Intrinsic {}
impl GetSpan for Expression {}
impl GetSpan for BlockExpression {}
