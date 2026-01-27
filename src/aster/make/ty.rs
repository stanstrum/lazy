use crate::lang::module::ModuleId;
use crate::line_dbg;
use crate::tokenize::token::Operator;

use super::*;

fn make_qualified<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
) -> Result<Option<lang::ty::Qualified>, Error> {
  let ret_mark = stream.mark();

  let start = stream.here()?;
  let implicit = {
    if let Some((Token::Operator(Operator::DoubleColon), _)) = stream.peek()? {
      stream.seek();
      true
    } else {
      false
    }
  };

  let mut mark;
  let mut parts = vec![];
  let mut expected = false;
  loop {
    mark = stream.mark();

    stream.skip_whitespace_and_comments()?;
    let Some(name) = make_name(lazy, stream)? else {
      if expected {
        return stream.expected_here(line_dbg!("an identifier"));
      } else {
        stream.take_mark(mark);
        break;
      };
    };

    parts.push(name);

    mark = stream.mark();
    stream.skip_whitespace_and_comments()?;
    let Some((Token::Operator(Operator::DoubleColon), _)) = stream.ok_next()? else {
      stream.take_mark(mark);
      break;
    };

    expected = true;
  };

  if !implicit && parts.is_empty() {
    stream.take_mark(ret_mark);
    return Ok(None);
  };

  let mut span = start;
  if let Some(last) = parts.last() {
    span.extend(last.span);
  };

  Ok(Some(lang::ty::Qualified {
    implicit,
    parts,
    span,
  }))
}

pub(super) fn make_type<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: ModuleId,
) -> Result<Option<lang::ty::Type>, Error> {
  Ok(if let Some(qualified) = make_qualified(lazy, stream)? {
    Some(lang::ty::Type::Unresolved { module, qualified })
  } else {
    None
  })
}
