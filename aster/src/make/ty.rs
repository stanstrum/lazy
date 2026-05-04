use lazy_macros::line_dbg;
use lang::token::{GroupingKind, GroupingType, Keyword, NumericValue, Operator};
use lang::span::GetSpan;
use lang::expr::LiteralKind;
use lang::module::AddTypePart;
use lang::Compiler;

use super::*;

pub(super) fn make_qualified<'pool, C: Compiler, const N: usize, T: Read>(
  stream: &mut Rereader<'pool, C, N, T>,
  module: C::ModuleReference,
) -> Result<Option<lang::ty::Qualified<C>>, Error<C>> {
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
    // mark = stream.mark();

    // stream.skip_whitespace_and_comments()?;
    let Some(name) = make_name(stream)? else {
      if expected {
        return stream.expected_here(line_dbg!("an identifier"));
      } else {
        // stream.take_mark(mark);
        break;
      };
    };

    parts.push(name);

    mark = stream.mark();
    // stream.skip_whitespace_and_comments()?;
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
    implicit: if implicit {
      lang::ty::QualifiedSearchSpace::Implicit
    } else {
      lang::ty::QualifiedSearchSpace::Module(module)
    },
    parts,
    span,
  }))
}

fn make_reference_to<'pool, C: Compiler, const N: usize, T: Read>(
  store: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, C, N, T>,
  module: C::ModuleReference,
) -> Result<Option<lang::ty::TypeValue<C>>, Error<C>> {
  let Some((Token::Operator(Operator::SingleAnd), mut span)) = stream.peek()? else {
    return Ok(None);
  };
  stream.seek();

  let r#mut = if let Some((Token::Keyword(Keyword::Mut), _)) = stream.peek()? {
    stream.seek();
    true
  } else {
    false
  };

  stream.skip_whitespace_and_comments()?;

  let Some(part_value) = make_type(store, stream, module)? else {
    return Ok(None);
  };

  let ty = module.add_type_part(part_value, store);

  span.extend(ty.get_span(store));

  Ok(Some(lang::ty::TypeValue::ReferenceTo { ty, r#mut, span, }))
}

fn make_array_of<'pool, C: Compiler, const N: usize, T: Read>(
  store: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, C, N, T>,
  module: C::ModuleReference,
) -> Result<Option<lang::ty::TypeValue<C>>, Error<C>> {
  let Some((Token::Grouping(GroupingType::Open(GroupingKind::Bracket)), start)) = stream.peek()? else {
    return Ok(None);
  };
  stream.seek();

  stream.skip_whitespace_and_comments()?;

  let size = if let Some((value, _, lit_span)) = expr::make_literal(store, stream)? {
    let LiteralKind::Numeric(NumericValue::U64(size)) = value else {
      return Err(Error::Invalid {
        what: line_dbg!("literal: must be an integer"),
        at: lit_span,
      });
    };

    stream.skip_whitespace_and_comments()?;

    Some(size as usize)
  } else {
    None
  };

  let Some((Token::Grouping(GroupingType::Close(GroupingKind::Bracket)), _)) = stream.peek()? else {
    return stream.expected_here(line_dbg!("a closing bracket"));
  };
  stream.seek();

  stream.skip_whitespace_and_comments()?;

  let Some(ty) = make_type(store, stream, module)? else {
    return stream.expected_here(line_dbg!("a type"));
  };

  let ty = module.add_type_part(ty, store);

  let mut span = start;
  span.extend(ty.get_span(store));

  Ok(Some(match size {
    Some(size) => lang::ty::TypeValue::SizedArrayOf { ty, size, span },
    None => lang::ty::TypeValue::UnsizedArrayOf { ty, span }
  }))
}

pub(super) fn make_type<'pool, C: Compiler, const N: usize, T: Read>(
  lazy: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, C, N, T>,
  module: C::ModuleReference,
) -> Result<Option<lang::ty::TypeValue<C>>, Error<C>> {
  if let Some(qualified) = make_qualified(stream, module)? {
    return Ok(Some(lang::ty::TypeValue::Unresolved { module, qualified }));
  };

  if let Some(reference_to) = make_reference_to(lazy, stream, module)? {
    return Ok(Some(reference_to));
  };

  if let Some(array_of) = make_array_of(lazy, stream, module)? {
    return Ok(Some(array_of));
  };

  Ok(None)
}
