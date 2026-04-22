use crate::print_once_per_thread;
use crate::tokenize::token::{GroupingKind, GroupingType, Keyword, Operator};
use crate::lang::expr::operator::{BinaryOperator, UnaryPrefixOperator, UnarySuffixOperator};

use super::*;

pub(super) fn make_unary_prefix<'pool, const N: usize, T: Read>(
  stream: &mut Rereader<'pool, N, T>,
) -> Result<Option<(UnaryPrefixOperator, Span)>, Error> {
  let Some((Token::Operator(token), mut span)) = stream.peek()? else {
    return Ok(None);
  };

  let op = match token {
    | Operator::Range
    | Operator::RightArrow
    | Operator::DoubleColon
    | Operator::Semicolon
    | Operator::Bollocks
    | Operator::Comma
    | Operator::Div
    | Operator::Mod
    | Operator::Or
    | Operator::Xor
    | Operator::OrAssign
    | Operator::AndAssign
    | Operator::XorAssign
    | Operator::LogicalOr
    | Operator::LogicalAnd
    | Operator::LogicalXor
    | Operator::LogicalOrAssign
    | Operator::LogicalAndAssign
    | Operator::LogicalXorAssign
    | Operator::Dot
    | Operator::Colon
    | Operator::Shl
    | Operator::Shr
    | Operator::ShlAssign
    | Operator::ShrAssign
    | Operator::LogicalShrAssign
    | Operator::Greater
    | Operator::GreaterEqual
    | Operator::Less
    | Operator::LessEqual
    | Operator::Equal
    | Operator::Assign
    | Operator::LogicalShr
    | Operator::AddAssign
    | Operator::SubAssign
    | Operator::MulAssign
    | Operator::ExpAssign
    | Operator::DivAssign
    | Operator::ModAssign
      => return Ok(None),
    Operator::Plus => UnaryPrefixOperator::Identity,
    Operator::SingleAnd => {
      stream.seek();

      if let Some((Token::Keyword(Keyword::Mut), end)) = stream.peek()? {
        span.extend(end);

        UnaryPrefixOperator::MutRef
      } else {
        UnaryPrefixOperator::Ref
      }
    },
    Operator::Minus => UnaryPrefixOperator::Negate,
    Operator::Asterisk => UnaryPrefixOperator::Deref,
    Operator::DoublePlus => UnaryPrefixOperator::PreDecrement,
    Operator::DoubleMinus => UnaryPrefixOperator::PreIncrement,
    Operator::Splat => UnaryPrefixOperator::Splat,
    Operator::Not => UnaryPrefixOperator::Not,
    Operator::Invert => UnaryPrefixOperator::Invert,
  };

  stream.seek();

  Ok(Some((op, span)))
}

pub(super) fn make_unary_suffix<'pool, const N: usize, T: Read>(
  lazy: &mut lazy::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: lang::reference::ModuleReference,
  block: lang::reference::BlockReference,
) -> Result<Option<(UnarySuffixOperator, Span)>, Error> {
  if let Some((Token::Operator(Operator::DoublePlus), span)) = stream.peek()? {
    stream.seek();
    return Ok(Some((UnarySuffixOperator::PostIncrement, span)));
  };

  if let Some((Token::Operator(Operator::DoubleMinus), span)) = stream.peek()? {
    stream.seek();
    return Ok(Some((UnarySuffixOperator::PostDecrement, span)));
  };

  print_once_per_thread!(lazy, {
    level: Stub,
    force: false,
    description: line_dbg!("parse cast").into(),
    contents: MessageContents::WithinSource(WithinSource::new(
      vec![MessageSection {
        text: "here".into(),
        span: stream.here()?,
      }]
    )),
  });

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

      let Some(expr) = make_expr(lazy, stream, module, block)? else {
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
//   lazy: &mut lazy::Lazy<'pool>,
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
  _lazy: &mut lazy::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  _module: lang::reference::ModuleReference,
  _function: lang::reference::FunctionReference,
) -> Result<Option<(BinaryOperator, Span)>, Error> {
  let Some((Token::Operator(token), mut span)) = stream.peek()? else {
    return Ok(None);
  };

  let op = match token {
    | Operator::DoubleColon
    | Operator::Semicolon
    | Operator::Bollocks
    | Operator::Comma
    | Operator::DoublePlus
    | Operator::DoubleMinus
    | Operator::Splat
    | Operator::Colon
    | Operator::Not
    | Operator::Invert
      => return Ok(None),
    Operator::Plus => BinaryOperator::Add,
    Operator::Minus => BinaryOperator::Sub,
    Operator::Asterisk => {
      let mark = stream.mark();
      stream.seek();

      if let Some((Token::Operator(Operator::Asterisk), end)) = stream.peek()? {
        span.extend(end);
        BinaryOperator::Exp
      } else {
        stream.take_mark(mark);
        BinaryOperator::Mul
      }
    },
    Operator::Div => BinaryOperator::Div,
    Operator::Range => BinaryOperator::Range,
    Operator::SingleAnd => BinaryOperator::And,
    Operator::Mod => BinaryOperator::Mod,
    Operator::Or => BinaryOperator::Or,
    Operator::Xor => BinaryOperator::Xor,
    Operator::OrAssign => BinaryOperator::OrAssign,
    Operator::AndAssign => BinaryOperator::AndAssign,
    Operator::XorAssign => BinaryOperator::XorAssign,
    Operator::LogicalOr => BinaryOperator::LogicalOr,
    Operator::LogicalAnd => BinaryOperator::LogicalAnd,
    Operator::LogicalXor => BinaryOperator::LogicalXor,
    Operator::LogicalShr => BinaryOperator::LogicalShr,
    Operator::LogicalOrAssign => BinaryOperator::LogicalOrAssign,
    Operator::LogicalAndAssign => BinaryOperator::LogicalAndAssign,
    Operator::LogicalXorAssign => BinaryOperator::LogicalXorAssign,
    Operator::Dot => BinaryOperator::Dot,
    Operator::RightArrow => BinaryOperator::DerefDot,
    Operator::Shl => BinaryOperator::Shl,
    Operator::Shr => BinaryOperator::Shr,
    Operator::ShlAssign => BinaryOperator::ShlAssign,
    Operator::ShrAssign => BinaryOperator::ShrAssign,
    Operator::LogicalShrAssign => BinaryOperator::LogicalShrAssign,
    Operator::Greater => BinaryOperator::Greater,
    Operator::GreaterEqual => BinaryOperator::GreaterEqual,
    Operator::Less => BinaryOperator::Less,
    Operator::LessEqual => BinaryOperator::LessEqual,
    Operator::Equal => BinaryOperator::Equal,
    Operator::Assign => BinaryOperator::Assign,
    Operator::AddAssign => BinaryOperator::AddAssign,
    Operator::SubAssign => BinaryOperator::SubAssign,
    Operator::MulAssign => BinaryOperator::MulAssign,
    Operator::ExpAssign => BinaryOperator::ExpAssign,
    Operator::DivAssign => BinaryOperator::DivAssign,
    Operator::ModAssign => BinaryOperator::ModAssign,
  };

  stream.seek();

  Ok(Some((op, span)))
}
