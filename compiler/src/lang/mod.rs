pub mod reference;
pub mod module;
pub mod function;
pub mod ty;
pub mod expr;

use std::path::PathBuf;

#[derive(Debug)]
pub enum LazyError {
  NotExist(PathBuf),
  Aster(crate::aster::Error),
}

impl ::lang::span::GetSpan<crate::lazy::LazyStructures> for crate::lang::expr::Expression {
  fn get_span(&self, lazy: &crate::Lazy) -> crate::tokenize::token::Span {
    match self {
      Self::Block(id) => id.get_span(lazy),
      | Self::Literal { span, .. }
      | Self::Variable { span, .. }
      | Self::Binary { span, .. }
      | Self::Unary { span, .. }
      | Self::StructInitializer { span, .. }
        => *span,
      Self::Unknown { qualified, .. } => qualified.span,
    }
  }
}
