use lang::span::GetSpan;
use lang::reference::Reference;

use crate::Lazy;
use crate::lazy::LazyStructures;
use crate::tokenize::token::Span;

impl GetSpan<LazyStructures> for crate::lang::module::Name {
  fn get_span(&self, _store: &Lazy) -> Span {
    self.span
  }
}

impl GetSpan<LazyStructures> for crate::lang::module::import::ImportPart {
  fn get_span(&self, _lazy: &Lazy) -> Span {
    match self {
      Self::Star(span) => *span,
      Self::Group(group) => group.span,
      Self::Qualify(qualify) => qualify.span,
    }
  }
}

impl GetSpan<LazyStructures> for crate::lang::ty::Type {
  fn get_span(&self, lazy: &Lazy) -> Span {
    match self {
      Self::Unresolved { qualified, .. } => qualified.span,
      | Self::Resolved { span, .. }
      | Self::ReferenceTo { span, .. }
      | Self::SizedArrayOf { span, .. }
      | Self::UnsizedArrayOf { span, .. }
      | Self::Intrinsic { span, .. }
      | Self::Weak { span }
      | Self::WeakInteger { span }
      | Self::WeakFloat { span }
      | Self::WeakString { span, .. }
        => *span,
      Self::Reference(reference) => reference.get_span(lazy),
      Self::Struct { prototype } => prototype.rget_from(lazy).span,
    }
  }
}

impl GetSpan<LazyStructures> for crate::resolve::TypePair {
  fn get_span(&self, store: &Lazy<'_>) -> Span {
    self.overwrite.get_span(store)
  }
}

impl GetSpan<LazyStructures> for crate::resolve::tasks::OverwriteTypeReference {
  fn get_span(&self, store: &crate::Lazy<'_>) -> Span {
    self.reference.get_span(store)
  }
}

impl GetSpan<LazyStructures> for crate::lang::expr::Expression {
  fn get_span(&self, lazy: &Lazy) -> Span {
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

impl GetSpan<LazyStructures> for crate::lang::expr::BlockExpression {
  fn get_span(&self, _lazy: &Lazy) -> Span {
    self.span
  }
}

impl GetSpan<LazyStructures> for crate::lang::expr::Variable {
  fn get_span(&self, _store: &Lazy<'_>) -> Span {
    self.span
  }
}
