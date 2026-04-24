use crate::tokenize::token::Span;
use crate::Lazy;
use ::lang::span::GetSpan;
use crate::lang::module::Name;
use crate::lang::reference::ModuleReference;

#[derive(Debug)]
pub struct ImportGroup {
  pub selectors: Vec<ImportPart>,
  pub span: Span,
}

#[derive(Debug)]
pub struct ImportQualify {
  pub name: Name,
  pub next: Option<Box<ImportPart>>,
  pub span: Span,
}

#[derive(Debug)]
pub enum ImportPart {
  Star(Span),
  Group(ImportGroup),
  Qualify(ImportQualify),
}

#[derive(Debug)]
pub struct Import {
  /// The imported module, i.e. `module` in the following snippet:
  ///
  ///     import from "module"
  ///       foo::bar::*
  pub source: ModuleReference,
  pub group: ImportGroup,
  pub span: Span,
}

impl ImportQualify {
  pub fn new(name: Name) -> Self {
    Self {
      name,
      next: None,
      span: name.span,
    }
  }
}

impl GetSpan<crate::lazy::LazyStructures> for ImportPart {
  fn get_span(&self, _lazy: &Lazy) -> Span {
    match self {
      ImportPart::Star(span) => *span,
      ImportPart::Group(group) => group.span,
      ImportPart::Qualify(qualify) => qualify.span,
    }
  }
}
