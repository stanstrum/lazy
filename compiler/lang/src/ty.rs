use crate::{Compiler, StringKind, intrinsic::Intrinsic, span::Span};

#[derive(Debug, Clone)]
pub enum QualifiedSearchSpace<C: Compiler> {
  Implicit,
  Struct(C::StructReference),
  Type(C::OverwriteTypeReference),
  Intrinsic {
    kind: crate::intrinsic::Intrinsic,
    span: crate::span::Span<C>,
  },
  Module(C::ModuleReference),
}

#[derive(Debug, Clone)]
pub struct Qualified<C: Compiler> {
  pub implicit: crate::ty::QualifiedSearchSpace<C>,
  pub parts: Vec<crate::module::Name<C>>,
  pub span: crate::span::Span<C>,
}

impl<C: Compiler> Qualified<C> {
  pub fn is_implicit(&self) -> bool {
    matches!(&self.implicit, QualifiedSearchSpace::Implicit)
  }
}

#[derive(Debug, Clone)]
pub enum Type<C: Compiler> {
  Reference(C::TypeReference),
  Resolved {
    part: C::TypePartReference,
    span: Span<C>,
  },
  Unresolved {
    module: C::ModuleReference,
    qualified: Qualified<C>,
  },
  Intrinsic {
    kind: Intrinsic,
    span: Span<C>,
  },
  // Resolved {
  //   original: Box<Type>,
  //   reference: TypeReference,
  // },
  WeakInteger {
    span: Span<C>,
  },
  WeakFloat {
    span: Span<C>,
  },
  WeakString {
    kind: StringKind,
    characters: usize,
    span: Span<C>,
    dereferenced: bool,
  },
  Weak {
    span: Span<C>,
  },
  ReferenceTo {
    ty: C::TypePartReference,
    r#mut: bool,
    span: Span<C>,
  },
  UnsizedArrayOf {
    ty: C::TypePartReference,
    span: Span<C>,
  },
  SizedArrayOf {
    ty: C::TypePartReference,
    size: usize,
    span: Span<C>,
  },
  Struct {
    prototype: C::StructReference,
  },
}
