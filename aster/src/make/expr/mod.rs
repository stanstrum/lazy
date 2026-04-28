pub mod variable;
pub mod block;
mod operator;
mod pemdas;
mod initializer;

use std::io::Read;

use lang::{Compiler, CompilerPoolStore};
use lang::span::Span;

use crate::make::ty;
use crate::Rereader;
use lazy_macros::line_dbg;
use lang::expr::LiteralKind;
use lang::reference::{BlockReference, ExpressionReference, Reference};
use lang::token::Token;

use super::Error;

fn new_weak_string<C: Compiler>(
  store: &C::Store<'_>,
  kind: lang::token::StringKind,
  value: string_pool::StringId,
  span: Span<C>,
) -> lang::ty::TypeValue<C> {
  let length = unsafe { store.pool().get_string(value).len() };

  let characters = match kind {
    lang::token::StringKind::C => length + 1,
    _ => length,
  };

  lang::ty::TypeValue::WeakString {
    kind,
    characters,
    span,
    dereferenced: false,
  }
}

pub(super) fn make_literal<'pool, C: Compiler, const N: usize, T: Read>(
  store: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, C, N, T>,
) -> Result<Option<lang::expr::Expression<C>>, Error<C>> {
  if let Some((Token::Numeric(value), span)) = stream.peek()? {
    stream.seek();

    let out = match value {
      lang::token::NumericValue::U64(_) => lang::ty::TypeValue::WeakInteger { span },
      lang::token::NumericValue::F64(_) => lang::ty::TypeValue::WeakFloat { span },
    };

    let value = lang::expr::LiteralKind::Numeric(value);

    return Ok(Some(lang::expr::Expression::Literal { value, span, out }));
  };

  if let Some((Token::String(kind, value), span)) = stream.peek()? {
    stream.seek();

    let out = new_weak_string(store, kind, value, span);
    let value = LiteralKind::String { kind, value };

    return Ok(Some(lang::expr::Expression::Literal { value, span, out }));
  };

  Ok(None)
}

fn make_expr_part<'pool, C: Compiler, const N: usize, T: Read>(
  store: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, C, N, T>,
  module: C::ModuleReference,
  block: BlockReference<C>,
) -> Result<Option<ExpressionReference<C>>, Error<C>> {
  let function = block.0;

  let expr = 'expr: {
    if let Some(initializer) = initializer::make_struct_initializer(store, stream, module, block)? {
      break 'expr initializer;
    };

    if let Some(block) = block::make_block(store, stream, module, function, Some(block))? {
      break 'expr lang::expr::Expression::Block(block);
    };

    if let Some(literal) = make_literal(store, stream)? {
      break 'expr literal;
    };

    if let Some(qualified) = ty::make_qualified(stream, module)? {
      break 'expr lang::expr::Expression::new_unknown(qualified);
    };

    return Ok(None);
  };

  let id = function.rget_from_mut(store).add_expr(expr);
  let reference = ExpressionReference(block, id);

  Ok(Some(reference))
}

#[derive(Debug)]
enum ExpressionPart<C: Compiler> {
  UnaryPrefix((lang::expr::operator::UnaryPrefixOperator, Span<C>)),
  UnarySuffix((lang::expr::operator::UnarySuffixOperator<C>, Span<C>)),
  Binary((lang::expr::operator::BinaryOperator, Span<C>)),
  Expression(ExpressionReference<C>),
}

pub(super) fn make_expr<'pool, C: Compiler, const N: usize, T: Read>(
  store: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, C, N, T>,
  module: C::ModuleReference,
  block: BlockReference<C>,
) -> Result<Option<ExpressionReference<C>>, Error<C>> {
  let mut parts: Vec<ExpressionPart<C>> = vec![];
  let mut expect = false;

  let function = block.0;

  loop {
    let curr_mark = stream.mark();
    stream.skip_whitespace_and_comments()?;

    while let Some(prefix) = operator::make_unary_prefix(stream)? {
      parts.push(ExpressionPart::UnaryPrefix(prefix));
      stream.skip_whitespace_and_comments()?;
    };

    let Some(expr) = make_expr_part(store, stream, module, block)? else {
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

    while let Some(suffix) = operator::make_unary_suffix(store, stream, module, block)? {
      parts.push(ExpressionPart::UnarySuffix(suffix));
      suffix_mark = stream.mark();
      stream.skip_whitespace_and_comments()?;
    };

    let Some(binary) = operator::make_binary_op(store, stream, module, function)? else {
      stream.take_mark(suffix_mark);
      break;
    };

    parts.push(ExpressionPart::Binary(binary));
    expect = true;
  };

  assert!(!parts.is_empty());
  // stream.take_mark(ret_mark);
  // return Ok(None);

  let reference = pemdas::melt(store, parts)?;

  Ok(Some(reference))
}
