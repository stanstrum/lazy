use std::io::Read;

use crate::{lang, line_dbg};
use crate::aster::Rereader;
use crate::tokenize::token::{self, GroupingKind, GroupingType, Token};

use super::Error;

fn make_block<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  parent: &mut lang::function::Function,
) -> Result<Option<lang::expr::BlockExpression>, Error> {
  let ret_mark = stream.mark();

  let Some((Token::Grouping(GroupingType::Open(GroupingKind::Brace)), temp_span)) = stream.peek()? else {
    return Ok(None);
  };

  stream.seek();
  stream.skip_whitespace_and_comments()?;

  let mut block = lang::expr::BlockExpression::new(temp_span);

  match stream.peek()? {
    Some((Token::Grouping(GroupingType::Close(GroupingKind::Brace)), end_span)) => {
      stream.seek();
      block.span.extend(end_span);
      return Ok(Some(block));
    },
    Some((Token::Indent(1..), _)) => stream.seek(),
    Some((Token::Indent(..=0), _)) => todo!(),
    _other => {
      stream.take_mark(ret_mark);
      return Err(Error::Invalid {
        what: line_dbg!("block (expected close brace or newline)"),
        at: stream.here()?,
      });
    },
  };

  loop {
    stream.skip_whitespace_and_comments()?;

    if let Some((Token::Indent(indent_level), _)) = stream.peek()? {
      match indent_level {
        0 => {
          stream.seek();
          continue;
        },
        ..0 => {
          stream.seek();
          break;
        },
        _ => {
          return Err(Error::Invalid {
            what: line_dbg!("indent (expecte"),
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
  };

  let Some((Token::Grouping(GroupingType::Close(GroupingKind::Brace)), end_span)) = stream.peek()? else {
    return stream.expected_here(line_dbg!("close brace"));
  };
  stream.seek();

  block.span.extend(end_span);

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
