use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::lang::span::GetSpan;
use crate::lang::expr::operator::{BinaryOperator, UnaryOperator, UnaryPrefixOperator, UnarySuffixOperator};

use super::*;

#[derive(Debug, Clone, Copy, EnumIter)]
enum Pemdas {
  Dot,
  Call,
  RefDeref,
  Increment,
  // Parenthesis,
  IdentNegate,
  Exponent,
  MulDivMod,
  AddSub,
  Bit,
  BitShift,
  Compare,
  Assign,
}

fn debug_gspan(lazy: &lang::Lazy, part: &ExpressionPart) -> Span {
  match part {
    | &ExpressionPart::UnaryPrefix((_, span))
    | &ExpressionPart::UnarySuffix((_, span))
    | &ExpressionPart::Binary((_, span))
      => span,
    ExpressionPart::Expression(expr) => lang::span::GetSpan::get_span(expr.rget_from(lazy), lazy),
  }
}

fn find_left_expr(cursor: usize, parts: &[ExpressionPart]) -> Option<usize> {
  parts[..=cursor].iter().rposition(|part| matches!(part, ExpressionPart::Expression(_)))
}

fn find_right_expr(cursor: usize, parts: &[ExpressionPart]) -> Option<usize> {
  parts[cursor..].iter().position(|part| matches!(part, ExpressionPart::Expression(_)))
    .map(|offset| cursor + offset)
}

fn melt_left(lazy: &mut lang::Lazy, cursor: &mut usize, parts: &mut Vec<ExpressionPart>) -> Result<lang::reference::ExpressionReference, Error> {
  let left = find_left_expr(*cursor, parts).unwrap();
  let melt_start = left + 1;
  let melt_end = *cursor;

  assert!(melt_start <= melt_end);
  let range = melt_start..melt_end;
  let range_count = range.len();

  let &ExpressionPart::Expression(mut expr) = parts.get(left).unwrap() else {
    unreachable!();
  };
  let function = expr.0;

  for suffix in parts.drain(range) {
    let ExpressionPart::UnarySuffix((suffix, op_span)) = suffix else {
      unreachable!();
    };

    let op = (UnaryOperator::Suffix(suffix), op_span);

    let mut span = expr.rget_from(lazy).get_span(lazy);
    span.extend(op_span);

    let new_expr = lang::expr::Expression::Unary { expr, op, span };
    let new_id = function.rget_from_mut(lazy).add_expr(new_expr);
    expr = lang::reference::ExpressionReference(function, new_id);
  };

  *parts.get_mut(left).unwrap() = ExpressionPart::Expression(expr);
  *cursor -= range_count;

  Ok(expr)
}

fn melt_right(lazy: &mut lang::Lazy, cursor: usize, parts: &mut Vec<ExpressionPart>) -> Result<lang::reference::ExpressionReference, Error> {
  let right = find_right_expr(cursor, parts).unwrap();
  let melt_start = cursor;
  let melt_end = right;

  assert!(melt_start <= melt_end);
  let range = melt_start..melt_end;

  let &ExpressionPart::Expression(mut expr) = parts.get(right).unwrap() else {
    unreachable!();
  };
  let function = expr.0;

  for prefix in parts.drain(range).rev() {
    let ExpressionPart::UnaryPrefix((prefix, op_span)) = prefix else {
      unreachable!();
    };

    let op = (UnaryOperator::Prefix(prefix), op_span);

    let mut span = op_span;
    let expr_span = expr.rget_from(lazy).get_span(lazy);
    span.extend(expr_span);

    let new_expr = lang::expr::Expression::Unary { expr, op, span };
    let new_id = function.rget_from_mut(lazy).add_expr(new_expr);
    expr = lang::reference::ExpressionReference(function, new_id);
  };

  *parts.get_mut(cursor).unwrap() = ExpressionPart::Expression(expr);

  Ok(expr)
}

pub(crate) fn melt(lazy: &mut lang::Lazy, mut parts: Vec<ExpressionPart>) -> Result<lang::reference::ExpressionReference, Error> {
  for step in Pemdas::iter() {
    let mut i = 0;

    while i < parts.len() {
      let part = parts.get(i).unwrap();

      match (&step, part) {
        (Pemdas::Call, ExpressionPart::UnarySuffix((UnarySuffixOperator::Call(_), _))) => todo!("Call"),
        (Pemdas::RefDeref, ExpressionPart::UnaryPrefix((
          | UnaryPrefixOperator::Ref
          | UnaryPrefixOperator::MutRef
          | UnaryPrefixOperator::Deref
        , _))) => todo!("RefDeref"),
        (Pemdas::Increment, ExpressionPart::UnaryPrefix((
          | UnaryPrefixOperator::PreDecrement
          | UnaryPrefixOperator::PreIncrement
        , _))) => todo!("IncrementPre"),
        (Pemdas::Increment, ExpressionPart::UnarySuffix((
          | UnarySuffixOperator::PostDecrement
          | UnarySuffixOperator::PostIncrement
        , _))) => todo!("IncrementPost"),
        (Pemdas::IdentNegate, &ExpressionPart::UnaryPrefix((
          | UnaryPrefixOperator::Identity
          | UnaryPrefixOperator::Negate
          | UnaryPrefixOperator::Not
          | UnaryPrefixOperator::Invert
        , _))) => {
          melt_right(lazy, i, &mut parts)?;
        },
        | (Pemdas::Dot, &ExpressionPart::Binary(op @ (BinaryOperator::Dot, _)))
        | (Pemdas::Exponent, &ExpressionPart::Binary(op @ (BinaryOperator::Exp, _)))
        | (Pemdas::MulDivMod, &ExpressionPart::Binary(op @ (
          | BinaryOperator::Mul
          | BinaryOperator::Div
          | BinaryOperator::Mod
        , _)))
        | (Pemdas::AddSub, &ExpressionPart::Binary(op @ (
          | BinaryOperator::Add
          | BinaryOperator::Sub
        , _)))
        | (Pemdas::Bit, &ExpressionPart::Binary(op @ (
          | BinaryOperator::Or
          | BinaryOperator::And
          | BinaryOperator::Xor
        , _)))
        | (Pemdas::BitShift, &ExpressionPart::Binary(op @ (
          | BinaryOperator::Shl
          | BinaryOperator::Shr
          | BinaryOperator::LogicalShr
        , _)))
        | (Pemdas::Compare, &ExpressionPart::Binary(op @ (
          | BinaryOperator::Less | BinaryOperator::LessEqual
          | BinaryOperator::Greater | BinaryOperator::GreaterEqual
          | BinaryOperator::Equal
        , _)))
        | (Pemdas::Assign, &ExpressionPart::Binary(op @ (
          | BinaryOperator::Assign
          | BinaryOperator::AddAssign
          | BinaryOperator::SubAssign
          | BinaryOperator::MulAssign
          | BinaryOperator::DivAssign
          | BinaryOperator::ModAssign
          | BinaryOperator::ExpAssign
          | BinaryOperator::AndAssign
          | BinaryOperator::OrAssign
          | BinaryOperator::XorAssign
          | BinaryOperator::ShlAssign
          | BinaryOperator::ShrAssign
          | BinaryOperator::LogicalAndAssign
          | BinaryOperator::LogicalOrAssign
          | BinaryOperator::LogicalXorAssign
          | BinaryOperator::LogicalShrAssign
        , _)))
        => {
          let a = melt_left(lazy, &mut i, &mut parts)?;
          let b = melt_right(lazy, i + 1, &mut parts)?;

          let start = a.rget_from(lazy).get_span(lazy);
          let end = b.rget_from(lazy).get_span(lazy);
          let span = Span::from_pair(start, end);

          let function = a.0;

          let expr = lang::expr::Expression::Binary { a, b, op, span };
          let id = function.rget_from_mut(lazy).add_expr(expr);
          let reference = lang::reference::ExpressionReference(function, id);

          parts.drain(i - 1 ..= i + 1);
          i -= 1;
          parts.insert(i, ExpressionPart::Expression(reference));
        },
        | (Pemdas::Dot, _)
        | (Pemdas::Call, _)
        | (Pemdas::RefDeref, _)
        | (Pemdas::Increment, _)
        | (Pemdas::IdentNegate, _)
        | (Pemdas::Exponent, _)
        | (Pemdas::MulDivMod, _)
        | (Pemdas::AddSub, _)
        | (Pemdas::Bit, _)
        | (Pemdas::BitShift, _)
        | (Pemdas::Compare, _)
        | (Pemdas::Assign, _)
        => { /* do nothing */ },
      };

      i += 1;
    };
  };

  if parts.len() != 1 {
    panic!("{parts:#?}");
  };

  let first = parts.into_iter().next().unwrap();

  let ExpressionPart::Expression(reference) = first else {
    panic!("pemdas failure: {first:#?}")
  };

  Ok(reference)
}
