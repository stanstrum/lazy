pub mod variable;
pub mod block;
mod operator;

use std::io::Read;

use crate::tokenize::token::{self, Token, Span};
use crate::aster::make::ty;
use crate::aster::Rereader;
use crate::{lang, line_dbg};
use crate::lang::expr::LiteralKind;
use crate::lang::reference::Reference;

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

    let value = lang::expr::LiteralKind::Numeric(value);

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

fn make_expr_part<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: lang::reference::ModuleReference,
  function: lang::reference::FunctionReference,
) -> Result<Option<lang::reference::ExpressionReference>, Error> {
  let expr = 'expr: {
    if let Some(block) = block::make_block(lazy, stream, module, function)? {
      break 'expr lang::expr::Expression::Block(block);
    };

    if let Some(literal) = make_literal(lazy, stream)? {
      break 'expr literal;
    };

    if let Some(qualified) = ty::make_qualified(stream)? {
      break 'expr lang::expr::Expression::Unknown(qualified);
    };

    return Ok(None);
  };

  let id = function.rget_from_mut(lazy).add_expr(expr);
  let reference = lang::reference::ExpressionReference(function, id);

  Ok(Some(reference))
}

#[derive(Debug)]
enum ExpressionPart {
  UnaryPrefix((lang::expr::operator::UnaryPrefixOperator, Span)),
  UnarySuffix((lang::expr::operator::UnarySuffixOperator, Span)),
  Binary((lang::expr::operator::BinaryOperator, Span)),
  Expression(lang::reference::ExpressionReference),
}

pub(super) fn make_expr<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: lang::reference::ModuleReference,
  function: lang::reference::FunctionReference,
) -> Result<Option<lang::reference::ExpressionReference>, Error> {
  let mut parts: Vec<ExpressionPart> = vec![];
  let mut expect = false;

  loop {
    let curr_mark = stream.mark();
    stream.skip_whitespace_and_comments()?;

    while let Some(prefix) = operator::make_unary_prefix(lazy, stream, module, function)? {
      parts.push(ExpressionPart::UnaryPrefix(prefix));
      stream.skip_whitespace_and_comments()?;
    };

    let Some(expr) = make_expr_part(lazy, stream, module, function)? else {
      if expect {
        return stream.expected_here(line_dbg!("an expression part"));
      } else {
        stream.take_mark(curr_mark);
        return Ok(None);
      };
    };

    parts.push(ExpressionPart::Expression(expr));

    let mut suffix_mark = stream.mark();
    stream.skip_whitespace_and_comments()?;

    while let Some(suffix) = operator::make_unary_suffix(lazy, stream, module, function)? {
      parts.push(ExpressionPart::UnarySuffix(suffix));
      suffix_mark = stream.mark();
      stream.skip_whitespace_and_comments()?;
    };

    let Some(binary) = operator::make_binary_op(lazy, stream, module, function)? else {
      stream.take_mark(suffix_mark);
      break;
    };

    parts.push(ExpressionPart::Binary(binary));
    expect = true;
  };

  assert!(!parts.is_empty());
  // stream.take_mark(ret_mark);
  // return Ok(None);

  while parts.len() > 1 {
    todo!("{parts:#?}")
  };

  let first = parts.into_iter().next().unwrap();

  let ExpressionPart::Expression(reference) = first else {
    panic!("pemdas failure: {first:#?}")
  };

  Ok(Some(reference))
}
