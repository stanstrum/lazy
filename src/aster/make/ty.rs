use crate::lang::module::ModuleId;
use crate::lang::span::GetSpan;
use crate::line_dbg;
use crate::tokenize::token::Operator;

use super::*;

fn make_qualified<'pool, const N: usize, T: Read>(
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
    let Some(name) = make_name(stream)? else {
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

fn make_reference_to<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: ModuleId,
) -> Result<Option<lang::ty::Type>, Error> {
  let Some((Token::Operator(Operator::SingleAnd), mut span)) = stream.peek()? else {
    return Ok(None);
  };
  stream.seek();
  stream.skip_whitespace_and_comments()?;

  let Some(ty) = make_type(lazy, stream, module)?.map(Box::new) else {
    return stream.expected_here(line_dbg!("a type"))?;
  };

  span.extend(ty.get_span(lazy));

  Ok(Some(lang::ty::Type::ReferenceTo { ty, span }))
}

pub(super) fn make_type<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: ModuleId,
) -> Result<Option<lang::ty::Type>, Error> {
  if let Some(qualified) = make_qualified(stream)? {
    Ok(Some(lang::ty::Type::Unresolved { module, qualified }))
  } else if let Some(reference_to) = make_reference_to(lazy, stream, module)? {
    Ok(Some(reference_to))
  } else {
    Ok(None)
  }
}
