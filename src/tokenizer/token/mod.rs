mod span;
mod consts;
pub(crate) mod error;

pub(crate) use consts::*;
pub(crate) use span::*;

use crate::compiler::CompilerResult;
use error::*;

use crate::tokenizer::impls::numeric::NumericState;

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum NumericKind {
  Float(f64),
  Integer(u64),
  // ^ signs are not read-in at the tokenization stage, therefore they cannot
  // have a sign.  the sign gets processed at the AST stage and later gets
  // computed by the postprocessor.
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum TokenKind {
  Whitespace,
  Identifier(String),
  Operator(Operator),
  Keyword(Keyword),
  Comment(String),
  Punctuation(Punctuation),
  Grouping(Grouping),
  Numeric(NumericKind),
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Token {
  pub kind: TokenKind,
  pub span: Span,
}

impl NumericKind {
  pub fn from_state_and_content(state: super::impls::numeric::NumericState, content: &str) -> CompilerResult<Self> {
    let base = match state {
      NumericState::Binary => 2,
      NumericState::Octal => 8,
      NumericState::Decimal => 10,
      NumericState::Hexadecimal => 16,
    };

    Ok({
      if content.find('.').is_some() {
        let Ok(value) = content.parse() else {
          return InvalidSnafu { what: What::Float, content }.fail()?;
        };

        Self::Float(value)
      } else {
        let Ok(value) = u64::from_str_radix(content, base) else {
          return InvalidSnafu { what: What::Integer, content }.fail()?;
        };

        Self::Integer(value)
      }
    })
  }
}
