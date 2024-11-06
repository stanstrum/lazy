use super::*;

/// A simple type, i.e. non-arithmetic
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum Type<W: CompilerWorkflow> {
  /// A type that is only referred to by name
  Qualified(Qualified<W>),
}

/// A type alias, e.g.:
///
/// ```
/// type usize := u64;
/// ```
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct TypeAlias<W: CompilerWorkflow> {
  pub(crate) name: Identifier<W>,
  pub(crate) ty: Type<W>,
  pub(crate) span: Span<W>,
}
