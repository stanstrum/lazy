use snafu::prelude::*;

use crate::compiler::error::ReadSpan;

#[derive(Debug)]
pub(crate) enum What {
  TopLevelNamespace,
  Identifier,
  Type,
  FunctionArguments,
  FunctionBody,
  Expression,
  Semicolon,
  ClosingBrace,
  ExportChild,
  Punctuation,
}

impl What {
  #[allow(unused)]
  pub fn as_strs(&self) -> (Option<&str>, &str) {
    const A: Option<&str> = Some("a");
    const AN: Option<&str> = Some("an");
    const NULL: Option<&str> = None;

    match self {
      What::TopLevelNamespace => (A, "top-level namespace"),
      What::Identifier => (AN, "identifier"),
      What::Type => (A, "type"),
      What::FunctionArguments => (NULL, "function arguments"),
      What::FunctionBody => (NULL, "function body"),
      What::Expression => (AN, "expression"),
      What::Semicolon => (A, "semicolon"),
      What::ClosingBrace => (A, "closing brace"),
      What::ExportChild => (AN, "exported member"),
      What::Punctuation => (NULL, "punctuation"),
    }
  }

  pub fn as_definite(&self) -> String {
    let (article, name) = self.as_strs();

    if let Some(a_or_an) = article {
      format!("{a_or_an} {name}")
    } else {
      name.into()
    }
  }

  #[allow(unused)]
  pub fn as_name(&self) -> &str {
    let (_, name) = self.as_strs();

    name
  }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum AsterizerError {
  #[snafu(display("expected {}", what.as_definite()))]
  Expected {
    what: What,
    span: ReadSpan,
  },
}

impl crate::help::LazyHelp for AsterizerError {
  fn applicable_span(self) -> Option<ReadSpan> {
    match self {
      AsterizerError::Expected { span, .. } => Some(span),
    }
  }
}
