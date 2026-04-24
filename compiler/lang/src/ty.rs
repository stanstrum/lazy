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
