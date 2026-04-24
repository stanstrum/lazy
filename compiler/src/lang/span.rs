use crate::Lazy;
use crate::lang::reference::{Reference, Store};
use crate::tokenize::token::Span;
use crate::lang::expr::Expression;

pub trait GetSpan<C: lang::Compiler = crate::lazy::LazyStructures> {
  fn get_span(&self, store: &C::Store<'_>) -> lang::span::Span<C>;
}

impl<
  C: lang::Compiler,
  R: for<'a> Reference<C::Store<'a>>,
> GetSpan<C> for R
  where
    for<'a> C::Store<'a>: lang::reference::Store<R>,
    for<'a> <C::Store<'a> as lang::reference::Store<R>>::Out: GetSpan<C>,
{
  fn get_span(&self, store: &C::Store<'_>) -> lang::span::Span<C> {
    self.rget_from(store).get_span(store)
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
