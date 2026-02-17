use crate::tokenize::token::{GroupingKind, GroupingType, Keyword, Operator};
use crate::lang::expr::operator::{BinaryOperator, UnaryPrefixOperator, UnarySuffixOperator};

use super::*;

pub(super) fn make_unary_prefix<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: lang::reference::ModuleReference,
  function: lang::reference::FunctionReference,
) -> Result<Option<(UnaryPrefixOperator, Span)>, Error> {
  let Some((Token::Operator(token), mut span)) = stream.peek()? else {
    return Ok(None);
  };

  let op = match token {
    | Operator::Add
    | Operator::Range
    | Operator::RightArrow
    | Operator::DoubleColon
    | Operator::Semicolon
    | Operator::Bollocks
    | Operator::Comma
    => return Ok(None),
    Operator::SingleAnd => {
      stream.seek();

      if let Some((Token::Keyword(Keyword::Mut), end)) = stream.peek()? {
        span.extend(end);

        UnaryPrefixOperator::MutRef
      } else {
        UnaryPrefixOperator::Ref
      }
    },
  };

  stream.seek();

  Ok(Some((op, span)))
}

pub(super) fn make_unary_suffix<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: lang::reference::ModuleReference,
  function: lang::reference::FunctionReference,
) -> Result<Option<(UnarySuffixOperator, Span)>, Error> {
  if let Some((Token::Grouping(GroupingType::Open(GroupingKind::Parenthesis)), mut span)) = stream.peek()? {
    stream.seek();

    let mut exprs = vec![];

    loop {
      stream.skip_whitespace_and_comments()?;

      if let Some((Token::Grouping(GroupingType::Close(GroupingKind::Parenthesis)), end)) = stream.peek()? {
        stream.seek();
        span.extend(end);
        break;
      };

      let Some(expr) = make_expr(lazy, stream, module, function)? else {
        return stream.expected_here(line_dbg!("an expression"));
      };

      exprs.push(expr);

      stream.skip_whitespace_and_comments()?;

      if let Some((Token::Operator(Operator::Comma), _)) = stream.peek()? {
        continue;
      };

      let Some((Token::Grouping(GroupingType::Close(GroupingKind::Parenthesis)), end)) = stream.peek()? else {
        return stream.expected_here(line_dbg!("a comma"));
      };
      span.extend(end);
      stream.seek();
      break;
    };

    return Ok(Some((UnarySuffixOperator::Call(exprs), span)))
  };

  if let Some((Token::Grouping(GroupingType::Open(GroupingKind::Bracket)), mut _span)) = stream.peek()? {
    todo!("subscript")
  };

  Ok(None)
}

// pub(super) fn make_unary_op<'pool, const N: usize, T: Read>(
//   lazy: &mut lang::Lazy<'pool>,
//   stream: &mut Rereader<'pool, N, T>,
//   module: lang::reference::ModuleReference,
//   function: lang::reference::FunctionReference,
// ) -> Result<Option<(UnaryOperator, Span)>, Error> {
//   if let Some(prefix) = make_unary_prefix(lazy, stream, module, function)? {
//     return Ok(Some(UnaryOperator::Prefix(prefix)));
//   };

//   if let Some(suffix) = make_unary_suffix(lazy, stream, module, function)? {
//     return Ok(Some(UnaryOperator::Suffix(suffix)));
//   };

//   Ok(None)
// }

pub(super) fn make_binary_op<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: lang::reference::ModuleReference,
  function: lang::reference::FunctionReference,
) -> Result<Option<(BinaryOperator, Span)>, Error> {
  let Some((Token::Operator(token), span)) = stream.peek()? else {
    return Ok(None);
  };

  let op = match token {
    | Operator::RightArrow
    | Operator::DoubleColon
    | Operator::Semicolon
    | Operator::Bollocks
    | Operator::Comma
      => return Ok(None),
    Operator::Add => BinaryOperator::Add,
    Operator::Range => BinaryOperator::Range,
    Operator::SingleAnd => BinaryOperator::And,
  };

  stream.seek();

  Ok(Some((op, span)))
}
