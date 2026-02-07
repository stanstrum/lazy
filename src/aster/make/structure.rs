use crate::line_dbg;
use crate::lang::span::GetSpan;
use crate::tokenize::token::{Keyword, Operator};

use super::*;

#[derive(Debug)]
pub enum Structure {
  Function(lang::reference::FunctionReference),
  TypeAlias(lang::reference::AliasReference),
}

fn make_type_alias<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  parent: lang::reference::ModuleReference,
) -> Result<Option<lang::reference::AliasReference>, Error> {
  let Some((Token::Keyword(Keyword::Type), start_span)) = stream.peek()? else {
    return Ok(None);
  };
  stream.seek();

  if !stream.skip_whitespace_and_comments()? {
    return stream.expected_here(line_dbg!("whitespace"));
  };

  let Some(name) = make_name(stream)? else {
    return stream.expected_here(line_dbg!("a name"));
  };

  stream.skip_whitespace_and_comments()?;

  let Some((Token::Operator(Operator::Bollocks), _)) = stream.peek()? else {
    return stream.expected_here(line_dbg!("assignment operator (:=)"));
  };
  stream.seek();

  stream.skip_whitespace_and_comments()?;

  let Some(ty) = ty::make_type(lazy, stream, parent)? else {
    return stream.expected_here(line_dbg!("a type"));
  };

  let mut span = start_span;
  span.extend(ty.get_span(lazy));

  lazy.rget(parent);

  Ok(Some(todo!()))
}

pub(super) fn make_structure<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  parent: lang::reference::ModuleReference,
) -> Result<Option<Structure>, Error> {
  if let Some(function) = function::make_function(lazy, stream, parent)? {
    Ok(Some(Structure::Function(function)))
  } else if let Some(alias) = make_type_alias(lazy, stream, parent)? {
    Ok(Some(Structure::TypeAlias(alias)))
  } else {
    Ok(None)
  }
}
