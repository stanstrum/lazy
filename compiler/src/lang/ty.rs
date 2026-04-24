use crate::tokenize::token::Span;
use ::lang::span::GetSpan;
use crate::lang::reference::Reference;

pub type QualifiedSearchSpace = ::lang::ty::QualifiedSearchSpace<crate::lazy::LazyStructures>;
pub type Qualified = ::lang::ty::Qualified<crate::lazy::LazyStructures>;

pub type Type = ::lang::ty::Type<crate::lazy::LazyStructures>;

impl GetSpan<crate::lazy::LazyStructures> for Type {
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
