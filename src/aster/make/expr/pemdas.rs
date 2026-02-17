use crate::lang::{expr::operator::UnaryOperator, span::GetSpan};

use super::*;

fn debug_gspan(lazy: &lang::Lazy, part: &ExpressionPart) -> Span {
  match part {
    | &ExpressionPart::UnaryPrefix((_, span))
    | &ExpressionPart::UnarySuffix((_, span))
    | &ExpressionPart::Binary((_, span))
      => span,
    ExpressionPart::Expression(expr) => lang::span::GetSpan::get_span(expr.rget_from(lazy), lazy),
  }
}

pub(crate) fn melt(lazy: &mut lang::Lazy, mut parts: Vec<ExpressionPart>) -> Result<lang::reference::ExpressionReference, Error> {
  // "Melt" the unary suffixes into their expressions first.  This is done
  // left-to-right.
  let mut i = 0;
  loop {
    let find = parts[i..].iter()
      .position(|part| matches!(part, ExpressionPart::Expression(_)));

    let Some(expr_i) = find else { break };
    i = expr_i + 1;

    while matches!(parts.get(i), Some(ExpressionPart::UnarySuffix(..))) {
      let ExpressionPart::UnarySuffix((op, op_span)) = parts.remove(i) else {
        unreachable!();
      };

      let &ExpressionPart::Expression(expr) = parts.get(expr_i).unwrap() else {
        unreachable!();
      };

      let op = (UnaryOperator::Suffix(op), op_span);
      let span = Span::from_pair(expr.rget_from(lazy).get_span(lazy), op_span);
      let new_expr = lang::expr::Expression::Unary { expr, op, span };

      let lang::reference::ExpressionReference(function, _) = expr;
      let new_id = function.rget_from_mut(lazy).add_expr(new_expr);

      parts[expr_i] = ExpressionPart::Expression(lang::reference::ExpressionReference(function, new_id));
    };
  };

  while parts.len() > 1 {
    let first = parts.first().unwrap();
    let last = parts.last().unwrap();

    let start = debug_gspan(lazy, first);
    let end = debug_gspan(lazy, last);

    return Err(Error::Invalid {
      what: line_dbg!("can't parse"),
      at: Span::from_pair(start, end),
    });
  };

  assert!(parts.len() == 1);

  let first = parts.into_iter().next().unwrap();

  let ExpressionPart::Expression(reference) = first else {
    panic!("pemdas failure: {first:#?}")
  };

  Ok(reference)
}
