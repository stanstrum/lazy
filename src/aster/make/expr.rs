use std::cmp::Ordering;
use std::io::Read;

use crate::resolve::reference::ExpressionReference;
use crate::{lang, line_dbg};
use crate::aster::Rereader;
use crate::tokenize::token::{self, GroupingKind, GroupingType, Operator, Span, Token};

use super::Error;

fn make_block<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  function: lang::module::FunctionId,
) -> Result<Option<lang::function::BlockId>, Error> {
  let indenter = stream.indenter_here()?;

  let Some((Token::Grouping(GroupingType::Open(GroupingKind::Brace)), start)) = stream.peek()? else {
    return Ok(None)
  };

  stream.seek();
  stream.skip_whitespace_and_comments()?;

  stream.skip_whitespace_and_comments()?;

  if let Some((Token::Grouping(GroupingType::Close(GroupingKind::Brace)), end)) = stream.peek()? {
    stream.seek();

    let span = Span::from_pair(start, end);

    return Ok(Some(lazy[function].add_block(lang::expr::BlockExpression {
      children: vec![],
      span: Span::from_pair(start, end),
      returns_last: false,
      out: lang::ty::Type::Intrinsic { kind: lang::ty::Intrinsic::Void, span },
    })));
  };

  let Some((Token::Indent(0..), _)) = indenter.peek(stream)? else {
    return stream.expected_here(line_dbg!("a newline (positive indent)"));
  };
  stream.seek();

  let mut children = vec![];
  let mut non_return_last = None;
  loop {
    stream.skip_whitespace_and_comments()?;

    if let Some((Token::Indent(indent), _)) = indenter.peek(stream)? {
      match indent.cmp(&0) {
        Ordering::Less => {
          stream.seek();
          break;
        },
        Ordering::Equal => {
          stream.seek();
          continue;
        },
        Ordering::Greater => {
          return Err(Error::Invalid {
            what: line_dbg!("newline (positive indent)"),
            at: stream.here()?,
          });
        },
      };
    };

    let Some(expr) = make_expr(lazy, stream, function)? else {
      return stream.expected_here(line_dbg!("an expression"));
    };
    let id = lazy[function].add_expr(expr);
    children.push(id);

    indenter.peek(stream)?;

    stream.skip_whitespace_and_comments()?;

    if let Some((Token::Operator(Operator::Semicolon), _)) = indenter.peek(stream)? {
      stream.seek();
      non_return_last = Some(id);

      stream.skip_whitespace_and_comments()?;
    };

    stream.peek()?;
    let Some((Token::Indent(indent), _)) = indenter.peek(stream)? else {
      return stream.expected_here(line_dbg!("a newline"));
    };

    match indent.cmp(&0) {
      Ordering::Less => {
        stream.seek();
        break;
      },
      Ordering::Equal => {
        stream.seek();
      },
      Ordering::Greater => {
        return Err(Error::Invalid {
          what: line_dbg!("newline (positive indent)"),
          at: stream.here()?,
        });
      },
    };
  };

  stream.skip_whitespace_and_comments()?;

  let Some((Token::Grouping(GroupingType::Close(GroupingKind::Brace)), end)) = indenter.peek(stream)? else {
    return stream.expected_here(line_dbg!("a closing brace"));
  };
  stream.seek();

  let span = Span::from_pair(start, end);

  let returns_last = !non_return_last.is_some_and(
    |id| id == *children.last().unwrap()
  );

  let out = if returns_last {
    let &index = lazy[function][lazy[function].body].children.last().unwrap();
    let reference = ExpressionReference { function, index };
    lang::ty::Type::Expression(reference)
  } else {
    lang::ty::Type::Intrinsic {
      kind: lang::ty::Intrinsic::Void,
      span,
    }
  };

  Ok(Some(lazy[function].add_block(lang::expr::BlockExpression {
    children,
    span,
    returns_last,
    out,
  })))
}

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

    Ok(Some(lang::expr::Expression::Literal { value, span, out }))
  } else {
    Ok(None)
  }
}

pub(super) fn make_expr<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  function: lang::module::FunctionId,
) -> Result<Option<lang::expr::Expression>, Error> {
  if let Some(block) = make_block(lazy, stream, function)? {
    Ok(Some(lang::expr::Expression::BlockExpression(block)))
  } else if let Some(literal) = make_literal(lazy, stream)? {
    Ok(Some(literal))
  } else {
    Ok(None)
  }
}
