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

pub(crate) fn melt(lazy: &lang::Lazy, parts: Vec<ExpressionPart>) -> Result<lang::reference::ExpressionReference, Error> {
  while parts.len() > 1 {
    let first = parts.first().unwrap();
    let last = parts.last().unwrap();

    let start = debug_gspan(lazy, first);
    let end = debug_gspan(lazy, last);

    return Err(Error::Invalid {
      what: line_dbg!("can't parse"),
      at: Span::from_pair(start, end),
    });

    todo!("{parts:#?}")
  };

  let first = parts.into_iter().next().unwrap();

  let ExpressionPart::Expression(reference) = first else {
    panic!("pemdas failure: {first:#?}")
  };

  Ok(reference)
}
