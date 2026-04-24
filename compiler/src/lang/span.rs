use crate::Lazy;
use crate::tokenize::token::Span;
use ::lang::span::GetSpan;
use crate::lang::reference::Reference;
use crate::lang::expr::Expression;

impl GetSpan<crate::lazy::LazyStructures> for Expression {
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
