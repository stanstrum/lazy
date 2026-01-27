use std::cmp::Ordering;
use std::io::Read;

use crate::{lang, line_dbg};
use crate::aster::Rereader;
use crate::tokenize::token::{self, GroupingKind, GroupingType, Operator, Token};

use super::Error;

fn make_block<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  parent: &mut lang::function::Function,
) -> Result<Option<lang::expr::BlockExpression>, Error> {
  let indenter = stream.indenter_here()?;

  let Some((Token::Grouping(GroupingType::Open(GroupingKind::Brace)), span)) = stream.peek()? else {
    return Ok(None)
  };

  stream.seek();
  stream.skip_whitespace_and_comments()?;

  let mut block = lang::expr::BlockExpression::new(span);

  stream.skip_whitespace_and_comments()?;

  if let Some((Token::Grouping(GroupingType::Close(GroupingKind::Brace)), end)) = stream.peek()? {
    stream.seek();
    block.span.extend(end);

    return Ok(Some(block));
  };

  let Some((Token::Indent(0..), _)) = indenter.peek(stream)? else {
    return stream.expected_here(line_dbg!("a newline (positive indent)"));
  };
  stream.seek();

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

    let Some(expr) = make_expr(lazy, stream, parent)? else {
      return stream.expected_here(line_dbg!("an expression"));
    };
    let id = parent.add_expr(expr);
    block.children.push(id);

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

  block.span.extend(end);

  block.returns_last = !non_return_last.is_some_and(
    |id| id == *block.children.last().unwrap()
  );

  Ok(Some(block))
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
  parent: &mut lang::function::Function,
) -> Result<Option<lang::expr::Expression>, Error> {
  if let Some(block) = make_block(lazy, stream, parent)? {
    let block = parent.add_block(block);
    Ok(Some(lang::expr::Expression::BlockExpression(block)))
  } else if let Some(literal) = make_literal(lazy, stream)? {
    Ok(Some(literal))
  } else {
    Ok(None)
  }
}
