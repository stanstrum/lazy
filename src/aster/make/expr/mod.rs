pub mod variable;
pub mod block;
mod operator;
mod pemdas;
mod initializer;

use std::io::Read;

use crate::tokenize::token::{self, Token, Span};
use crate::aster::make::ty;
use crate::aster::Rereader;
use crate::{lang, lazy, line_dbg};
use crate::lang::expr::LiteralKind;
use crate::lang::reference::Reference;

use super::Error;

fn new_weak_string(
  lazy: &lazy::Lazy,
  kind: token::StringKind,
  value: string_pool::StringId,
  span: token::Span
) -> lang::ty::Type {
  let length = unsafe { lazy.pool.get_string(value).len() };

  let characters = match kind {
    token::StringKind::C => length + 1,
    _ => length,
  };

  lang::ty::Type::WeakString {
    kind,
    characters,
    span,
    dereferenced: false,
  }
}

pub(super) fn make_literal<'pool, const N: usize, T: Read>(
  lazy: &mut lazy::Lazy<'pool>,
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

    let out = new_weak_string(lazy, kind, value, span);
    let value = LiteralKind::String { kind, value };

    return Ok(Some(lang::expr::Expression::Literal { value, span, out }));
  };

  Ok(None)
}

fn make_expr_part<'pool, const N: usize, T: Read>(
  lazy: &mut lazy::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: lang::reference::ModuleReference,
  block: lang::reference::BlockReference,
) -> Result<Option<lang::reference::ExpressionReference>, Error> {
  let function = block.0;

  let expr = 'expr: {
    if let Some(initializer) = initializer::make_struct_initializer(lazy, stream, module, block)? {
      break 'expr initializer;
    };

    if let Some(block) = block::make_block(lazy, stream, module, function, Some(block))? {
      break 'expr lang::expr::Expression::Block(block);
    };

    if let Some(literal) = make_literal(lazy, stream)? {
      break 'expr literal;
    };

    if let Some(qualified) = ty::make_qualified(stream, module)? {
      break 'expr lang::expr::Expression::new_unknown(qualified);
    };

    return Ok(None);
  };

  let id = function.rget_from_mut(lazy).add_expr(expr);
  let reference = lang::reference::ExpressionReference(block, id);

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
  lazy: &mut lazy::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: lang::reference::ModuleReference,
  block: lang::reference::BlockReference,
) -> Result<Option<lang::reference::ExpressionReference>, Error> {
  let mut parts: Vec<ExpressionPart> = vec![];
  let mut expect = false;

  let function = block.0;

  loop {
    let curr_mark = stream.mark();
    stream.skip_whitespace_and_comments()?;

    while let Some(prefix) = operator::make_unary_prefix(stream)? {
      parts.push(ExpressionPart::UnaryPrefix(prefix));
      stream.skip_whitespace_and_comments()?;
    };

    let Some(expr) = make_expr_part(lazy, stream, module, block)? else {
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

    while let Some(suffix) = operator::make_unary_suffix(lazy, stream, module, block)? {
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

  let reference = pemdas::melt(lazy, parts)?;

  Ok(Some(reference))
}
