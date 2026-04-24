use crate::tokenize::token::{Span, StringKind};
use ::lang::intrinsic::Intrinsic;
use ::lang::span::GetSpan;
use crate::lang::module::Name;
use crate::lang::reference::{ModuleReference, Reference, StructReference, TypePartReference, TypeReference};

pub type QualifiedSearchSpace = ::lang::ty::QualifiedSearchSpace<crate::lazy::LazyStructures>;

#[derive(Debug, Clone)]
pub struct Qualified {
  pub implicit: QualifiedSearchSpace,
  pub parts: Vec<Name>,
  pub span: Span,
}

impl Qualified {
  pub fn is_implicit(&self) -> bool {
    matches!(&self.implicit, QualifiedSearchSpace::Implicit)
  }
}

#[derive(Debug, Clone)]
pub enum Type {
  Reference(TypeReference),
  Resolved {
    part: TypePartReference,
    span: Span,
  },
  Unresolved {
    module: ModuleReference,
    qualified: Qualified,
  },
  Intrinsic {
    kind: Intrinsic,
    span: Span,
  },
  // Resolved {
  //   original: Box<Type>,
  //   reference: TypeReference,
  // },
  WeakInteger {
    span: Span,
  },
  WeakFloat {
    span: Span,
  },
  WeakString {
    kind: StringKind,
    characters: usize,
    span: Span,
    dereferenced: bool,
  },
  Weak {
    span: Span,
  },
  ReferenceTo {
    ty: TypePartReference,
    r#mut: bool,
    span: Span,
  },
  UnsizedArrayOf {
    ty: TypePartReference,
    span: Span,
  },
  SizedArrayOf {
    ty: TypePartReference,
    size: usize,
    span: Span,
  },
  Struct {
    prototype: StructReference,
  },
}

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
