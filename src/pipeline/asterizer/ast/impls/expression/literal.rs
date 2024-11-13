use crate::asterizer::{ast::*, Ast, Asterizer};
use crate::compiler::Compiler;
use crate::tokenizer::{NumericKind, SpanStart, TokenKind};
use crate::{impl_ast, Result};

impl_ast!(Literal: (_, aster, start) => {
  let kind = match aster.reader.next_kind() {
    Some(TokenKind::Numeric(NumericKind::Integer(value))) => {
      LiteralKind::Numeric(NumericLiteral::Generic(*value))
    },
    Some(TokenKind::Numeric(NumericKind::Float(value))) => {
      LiteralKind::Numeric(NumericLiteral::Float(*value))
    },
    Some(TokenKind::String(string)) => {
      LiteralKind::String(StringLiteral::Generic(string.to_owned()))
    },
    _ => return Ok(None),
  };

  Ok(Some(Self {
    kind,
    span: aster.finish_span(start)
  }))
});
