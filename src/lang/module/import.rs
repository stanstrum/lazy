use crate::string_pool::StringId;

use crate::tokenize::token::Span;
use crate::lang::Lazy;
use crate::lang::span::GetSpan;
use crate::lang::module::Name;

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
  pub source: StringId,
  pub group: ImportGroup,
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

impl GetSpan for ImportPart {
  fn get_span(&self, _lazy: &Lazy) -> Span {
    match self {
      ImportPart::Star(span) => *span,
      ImportPart::Group(group) => group.span,
      ImportPart::Qualify(qualify) => qualify.span,
    }
  }
}
