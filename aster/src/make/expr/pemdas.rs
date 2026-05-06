use lang::expr::BlockExpression;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use lazy_macros::print_message;
use lang::span::GetSpan;
use lang::expr::operator::{BinaryOperator, UnaryOperator, UnaryPrefixOperator, UnarySuffixOperator};

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
  Logical,
  Assign,
}

fn debug_gspan<C: Compiler>(store: &C::Store<'_>, part: &ExpressionPart<C>) -> Span<C> {
  match part {
    | &ExpressionPart::UnaryPrefix((_, span))
    | &ExpressionPart::UnarySuffix((_, span))
    | &ExpressionPart::Binary((_, span))
      => span,
    ExpressionPart::Expression(expr) => expr.get_span(store),
  }
}

fn find_left_expr<C: Compiler>(cursor: usize, parts: &[ExpressionPart<C>]) -> Option<usize> {
  parts[..cursor].iter().rposition(|part| matches!(part, ExpressionPart::Expression(_)))
}

fn find_right_expr<C: Compiler>(cursor: usize, parts: &[ExpressionPart<C>]) -> Option<usize> {
  parts[cursor..].iter().position(|part| matches!(part, ExpressionPart::Expression(_)))
    .map(|offset| cursor + offset)
}

fn melt_left<C: Compiler>(store: &mut C::Store<'_>, cursor: &mut usize, parts: &mut Vec<ExpressionPart<C>>) -> Result<ExpressionReference<C>, Error<C>> {
  let left = find_left_expr(*cursor, parts).unwrap();
  let melt_start = left + 1;
  let melt_end = (*cursor).min(parts.len());

  assert!(melt_start <= melt_end);
  let range = melt_start..melt_end;
  let range_count = range.len();

  let &ExpressionPart::Expression(mut expr) = parts.get(left).unwrap() else {
    unreachable!();
  };
  let block = expr.0;
  let function = block.0;

  for suffix in parts.drain(range) {
    let ExpressionPart::UnarySuffix((suffix, op_span)) = suffix else {
      unreachable!();
    };

    let op = (UnaryOperator::Suffix(suffix), op_span);

    let mut span = expr.rget_from(store).get_span(store);
    span.extend(op_span);

    let new_expr = lang::expr::Expression::Unary {
      expr,
      op,
      span,
      out: todo!(),
      // lang::ty::TypeValue::Weak { span },
    };
    let new_id = function.rget_from_mut(store).add_expr(new_expr);
    expr = ExpressionReference(block, new_id);
  };

  *parts.get_mut(left).unwrap() = ExpressionPart::Expression(expr);
  *cursor -= range_count;

  Ok(expr)
}

fn melt_right<C: Compiler>(store: &mut C::Store<'_>, cursor: usize, parts: &mut Vec<ExpressionPart<C>>) -> Result<ExpressionReference<C>, Error<C>> {
  let right = find_right_expr(cursor, parts).unwrap();
  let melt_start = cursor;
  let melt_end = right;

  assert!(melt_start <= melt_end);
  let range = melt_start..melt_end;

  let &ExpressionPart::Expression(mut expr) = parts.get(right).unwrap() else {
    unreachable!();
  };
  let block = expr.0;
  let function = block.0;

  for prefix in parts.drain(range).rev() {
    let ExpressionPart::UnaryPrefix((prefix, op_span)) = prefix else {
      unreachable!();
    };

    let op = (UnaryOperator::Prefix(prefix), op_span);

    let mut span = op_span;
    let expr_span = expr.rget_from(store).get_span(store);
    span.extend(expr_span);

    let new_expr = lang::expr::Expression::Unary {
      expr,
      op,
      span,
      out: todo!(),
      // lang::ty::TypeValue::Weak { span },
    };
    let new_id = function.rget_from_mut(store).add_expr(new_expr);
    expr = ExpressionReference(block, new_id);
  };

  *parts.get_mut(cursor).unwrap() = ExpressionPart::Expression(expr);

  Ok(expr)
}

pub(crate) fn melt<C: Compiler>(store: &mut C::Store<'_>, mut parts: Vec<ExpressionPart<C>>) -> Result<ExpressionReference<C>, Error<C>> {
  for step in Pemdas::iter() {
    let mut i = 0;

    while i < parts.len() {
      let part = parts.get(i).unwrap();

      match (&step, part) {
        | (Pemdas::Call, ExpressionPart::UnarySuffix((UnarySuffixOperator::Call(_), _)))
        | (Pemdas::Increment, ExpressionPart::UnarySuffix((
          | UnarySuffixOperator::PostDecrement
          | UnarySuffixOperator::PostIncrement
          , _))) => {
            i += 1;
            melt_left(store, &mut i, &mut parts)?;
            i -= 1;
          },
          | (Pemdas::RefDeref, ExpressionPart::UnaryPrefix((
            | UnaryPrefixOperator::Ref
            | UnaryPrefixOperator::MutRef
            | UnaryPrefixOperator::Deref
          , _)))
          | (Pemdas::Increment, ExpressionPart::UnaryPrefix((
            | UnaryPrefixOperator::PreDecrement
            | UnaryPrefixOperator::PreIncrement
          , _)))
          | (Pemdas::IdentNegate, ExpressionPart::UnaryPrefix((
          | UnaryPrefixOperator::Identity
          | UnaryPrefixOperator::Negate
          | UnaryPrefixOperator::Not
          | UnaryPrefixOperator::Invert
        , _))) => {
          melt_right(store, i, &mut parts)?;
        },
        | (Pemdas::Dot, &ExpressionPart::Binary(op @ (
          | BinaryOperator::Dot
          | BinaryOperator::DerefDot
        , _)))
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
        | (Pemdas::Logical, &ExpressionPart::Binary(op @ (
          | BinaryOperator::LogicalAnd
          | BinaryOperator::LogicalOr
          | BinaryOperator::LogicalXor
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
          let a = melt_left(store, &mut i, &mut parts)?;
          let b = melt_right(store, i + 1, &mut parts)?;

          let start = a.rget_from(store).get_span(store);
          let end = b.rget_from(store).get_span(store);
          let span = Span::from_pair(start, end);

          let block = a.0;

          let reference = BlockExpression::create_new_expr_in(store, block, |expr| {
            let type_reference = lang::reference::TypeReference::Expression(expr);
            let type_value = lang::ty::TypeValue::Weak { span };

            let out = lang::ty::Type::new(type_reference, type_value);

            lang::expr::Expression::Binary {
              a,
              b,
              op,
              span,
              out,
            }
          });

          parts.drain(i - 1 ..= i + 1);
          i -= 1;
          parts.insert(i, ExpressionPart::Expression(reference));
        },
        (
        | Pemdas::Dot
        | Pemdas::Call
        | Pemdas::RefDeref
        | Pemdas::Increment
        | Pemdas::IdentNegate
        | Pemdas::Exponent
        | Pemdas::MulDivMod
        | Pemdas::AddSub
        | Pemdas::Bit
        | Pemdas::BitShift
        | Pemdas::Compare
        | Pemdas::Logical
        | Pemdas::Assign
        , _)
        => { /* do nothing */ },
      };

      i += 1;
    };
  };

  if parts.len() != 1 {
    assert!(!parts.is_empty());

    let start = debug_gspan(store, parts.first().unwrap());
    let end = debug_gspan(store, parts.last().unwrap());
    let range = Span::from_pair(start, end);

    print_message!(store, {
      level: Warn,
      force: false,
      description: format!(line_dbg!("{} parts"), parts.len()),
      contents: MessageContents::WithinSource(vec![WithinSource {
        range,
        sections: parts.iter().enumerate().map(|(i, part)| MessageSection {
          text: format!("part {i}"),
          span: debug_gspan(store, part),
        }).collect(),
      }]),
    });

    panic!("{parts:#?}");
  };

  let first = parts.into_iter().next().unwrap();

  let ExpressionPart::Expression(reference) = first else {
    panic!("pemdas failure: {first:#?}")
  };

  Ok(reference)
}
