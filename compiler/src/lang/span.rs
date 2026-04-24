use crate::Lazy;
use crate::lang::reference::{Reference, Store};
use crate::tokenize::token::Span;
use crate::lang::expr::Expression;

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

impl GetSpan for Expression {
  fn get_span(&self, lazy: &Lazy) -> Span {
    match self {
      Expression::Block(id) => id.rget_from(lazy).get_span(lazy),
      | Expression::Literal { span, .. }
      | Expression::Variable { span, .. }
      | Expression::Binary { span, .. }
      | Expression::Unary { span, .. }
      | Expression::StructInitializer { span, .. }
        => *span,
      Expression::Unknown { qualified, .. } => qualified.span,
    }
  }
}
