use crate::{Compiler, module::Name, span::Span};


#[derive(Debug)]
pub struct ImportGroup<C: Compiler> {
  pub selectors: Vec<ImportPart<C>>,
  pub span: Span<C>,
}

#[derive(Debug)]
pub struct ImportQualify<C: Compiler> {
  pub name: Name<C>,
  pub next: Option<Box<ImportPart<C>>>,
  pub span: Span<C>,
}

#[derive(Debug)]
pub enum ImportPart<C: Compiler> {
  Star(Span<C>),
  Group(ImportGroup<C>),
  Qualify(ImportQualify<C>),
}

#[derive(Debug)]
pub struct Import<C: Compiler> {
  /// The imported module, i.e. `module` in the following snippet:
  ///
  ///     import from "module"
  ///       foo::bar::*
  pub source: C::ModuleReference,
  pub group: ImportGroup<C>,
  pub span: Span<C>,
}

impl<C: Compiler> ImportQualify<C> {
  pub fn new(name: Name<C>) -> Self {
    Self {
      name,
      next: None,
      span: name.span,
    }
  }
}
