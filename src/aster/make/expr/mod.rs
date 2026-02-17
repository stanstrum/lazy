pub mod variable;
pub mod block;

use std::io::Read;

use crate::aster::make::ty;
use crate::lang::expr::LiteralKind;
use crate::{lang, line_dbg};
use crate::aster::Rereader;
use crate::tokenize::token::{self, Operator, Token};

use super::Error;

pub(super) fn make_literal<'pool, const N: usize, T: Read>(
  _lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
) -> Result<Option<lang::expr::Expression>, Error> {
  if let Some((Token::Numeric(value), span)) = stream.peek()? {
    stream.seek();

    let out = match value {
      token::NumericValue::U64(_) => lang::ty::Type::WeakInteger { span },
      token::NumericValue::F64(_) => lang::ty::Type::WeakFloat { span },
    };

    let value = LiteralKind::Numeric(value);

    return Ok(Some(lang::expr::Expression::Literal { value, span, out }));
  };

  if let Some((Token::String(kind, value), span)) = stream.peek()? {
    stream.seek();

    let out = lang::ty::Type::WeakString { span };
    let value = LiteralKind::String { kind, value };

    return Ok(Some(lang::expr::Expression::Literal { value, span, out }));
  };

  Ok(None)
}

pub(super) fn make_expr<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: lang::reference::ModuleReference,
  function: lang::reference::FunctionReference,
) -> Result<Option<lang::expr::Expression>, Error> {
  if let Some(block) = block::make_block(lazy, stream, module, function)? {
    return Ok(Some(lang::expr::Expression::Block(block)));
  };

  if let Some(literal) = make_literal(lazy, stream)? {
    return Ok(Some(literal));
  };

  if let Some(qualified) = ty::make_qualified(stream)? {
    return Ok(Some(lang::expr::Expression::Unknown(qualified)));
  };

  Ok(None)
}
