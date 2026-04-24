use lang::{reference::Reference, span::GetSpan};

use crate::{lazy::LazyStructures, tokenize::token::Span};
use crate::lang::{expr::Expression, ty::Type};

impl GetSpan<LazyStructures> for Expression {
  fn get_span(&self, lazy: &crate::Lazy) -> Span {
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

impl GetSpan<LazyStructures> for Type {
  fn get_span(&self, lazy: &crate::Lazy) -> Span {
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
      Type::Struct { prototype } => prototype.rget_from(lazy).span,
    }
  }
}
