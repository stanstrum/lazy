use crate::Compiler;

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
