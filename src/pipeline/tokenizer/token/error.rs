use snafu::prelude::*;

use crate::compiler::error::CompilerError;

#[derive(Debug)]
pub(crate) enum What {
  Identifier,
  Numeric,
  Float,
  Integer,
  Operator,
  MultilineComment,
  String,
  StringEscapeSequence,
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum TokenError {
  #[snafu(display("expected {}", what.as_definite()))]
  Expected { what: What },

  #[snafu(display("invalid {} {content:?}", what.as_name()))]
  Invalid { what: What, content: String },
}

impl From<TokenError> for CompilerError {
  fn from(err: TokenError) -> Self {
    Self::Token { err }
  }
}

impl What {
  pub fn as_strs(&self) -> (&str, &str) {
    const A: &str = "a";
    const AN: &str = "an";

    match self {
      What::Identifier => (AN, "identifier"),
      What::Numeric => (A, "numeric"),
      What::Operator => (AN, "operator"),
      What::Float => (A, "float"),
      What::Integer => (AN, "integer"),
      What::MultilineComment => (A, "multiline comment"),
      What::String => (A, "string literal"),
      What::StringEscapeSequence => (A, "string escape sequence"),
    }
  }

  pub fn as_definite(&self) -> String {
    let (a_or_an, name) = self.as_strs();

    format!("{a_or_an} {name}")
  }

  pub fn as_name(&self) -> &str {
    let (_, name) = self.as_strs();

    name
  }
}
