use crate::Compiler;

use crate::span::{GetSpan, Span};
use crate::reference::Reference;

impl<C: Compiler> GetSpan<C> for crate::module::Name<C> {
  fn get_span(&self, _store: &C::Store<'_>) -> Span<C> {
    self.span
  }
}

impl<C: Compiler> GetSpan<C> for crate::module::Struct<C> {
  fn get_span(&self, store: &<C as Compiler>::Store<'_>) -> Span<C> {
    self.span
  }
}

impl<C: Compiler> GetSpan<C> for crate::import::ImportPart<C> {
  fn get_span(&self, _store: &C::Store<'_>) -> Span<C> {
    match self {
      Self::Star(span) => *span,
      Self::Group(group) => group.span,
      Self::Qualify(qualify) => qualify.span,
    }
  }
}

impl<C: Compiler> GetSpan<C> for crate::ty::TypeValue<C> {
  fn get_span(&self, store: &C::Store<'_>) -> Span<C> {
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
      Self::Reference(reference) => reference.rget_from(store).get_span(store),
      Self::Struct { prototype } => prototype.rget_from(store).get_span(store),
    }
  }
}

impl<C: Compiler> GetSpan<C> for crate::expr::Expression<C> {
  fn get_span(&self, store: &C::Store<'_>) -> Span<C> {
    match self {
      Self::Block(id) => id.rget_from(store).get_span(store),
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

impl<C: Compiler> GetSpan<C> for crate::expr::BlockExpression<C> {
  fn get_span(&self, _store: &C::Store<'_>) -> Span<C> {
    self.span
  }
}

impl<C: Compiler> GetSpan<C> for crate::expr::Variable<C> {
  fn get_span(&self, _store: &C::Store<'_>) -> Span<C> {
    self.span
  }
}

impl<C: Compiler> GetSpan<C> for crate::reference::BlockReference<C> {
  fn get_span(&self, store: &<C as Compiler>::Store<'_>) -> Span<C> {
    self.rget_from(store).get_span(store)
  }
}

impl<C: Compiler> GetSpan<C> for crate::reference::ExpressionReference<C> {
  fn get_span(&self, store: &<C as Compiler>::Store<'_>) -> Span<C> {
    self.rget_from(store).get_span(store)
  }
}

impl<C: Compiler> GetSpan<C> for crate::reference::VariableReference<C> {
  fn get_span(&self, store: &<C as Compiler>::Store<'_>) -> Span<C> {
    self.rget_from(store).get_span(store)
  }
}

impl<C: Compiler> GetSpan<C> for crate::reference::TypeReference<C> {
  fn get_span(&self, store: &<C as Compiler>::Store<'_>) -> Span<C> {
    self.rget_from(store).get_span(store)
  }
}

impl<C: Compiler> GetSpan<C> for crate::reference::TypePartReference<C> {
  fn get_span(&self, store: &<C as Compiler>::Store<'_>) -> Span<C> {
    self.rget_from(store).get_span(store)
  }
}

impl<C: Compiler> GetSpan<C> for crate::ty::Type<C> {
  fn get_span(&self, store: &C::Store<'_>) -> Span<C> {
    if let Some(ty) = self.ty.as_ref() {
      return ty.get_span(store);
    };

    self.reference.get_span(store)
  }
}

impl<C: Compiler> GetSpan<C> for crate::ty::ResolvedType<C> {
  fn get_span(&self, store: &C::Store<'_>) -> Span<C> {
    self.ty.get_span(store)
  }
}
