use crate::lang::reference::{ExpressionReference, Reference};
use crate::lang::span::GetSpan;
use crate::line_dbg;

use crate::tokenize::token::Operator;

use super::*;

pub(super) fn make_function_argument<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  parent: lang::reference::ModuleReference,
) -> Result<Option<lang::expr::Variable>, Error> {
  let Some(ty) = ty::make_type(lazy, stream, parent)? else {
    return Ok(None);
  };

  if !stream.skip_whitespace_and_comments()? {
    return stream.expected_here(line_dbg!("whitespace"));
  };

  let Some(name) = make_name(stream)? else {
    return stream.expected_here(line_dbg!("an identifier"));
  };

  let mut span = ty.get_span(lazy);
  span.extend(name.span);

  Ok(Some(lang::expr::Variable {
    name,
    ty,
    span,
  }))
}

fn make_function_header<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  parent: lang::reference::ModuleReference,
) -> Result<Option<lang::function::FunctionHeader>, Error> {
  let Some(name) = make_name(stream)? else {
    return Ok(None);
  };

  stream.skip_whitespace_and_comments()?;

  let ret_ty = {
    if let Some((Token::Operator(Operator::RightArrow), _)) = stream.peek()? {
      stream.seek();
      stream.skip_whitespace_and_comments()?;

      let Some(ret_ty) = ty::make_type(lazy, stream, parent)? else {
        return stream.expected_here(line_dbg!("a return type"));
      };

      ret_ty
    } else {
      lang::ty::Type::Intrinsic {
        kind: lang::ty::Intrinsic::Void,
        span: name.span,
      }
    }
  };

  stream.skip_whitespace_and_comments()?;

  let Some((Token::Indent(1..), _)) = stream.peek()? else {
    return stream.expected_here(line_dbg!("a positive indent"));
  };
  stream.seek();

  let mut arguments = vec![];
  loop {
    if let Some((Token::Indent(0), _)) = stream.peek()? {
      stream.seek();
      break;
    };

    stream.skip_whitespace_and_comments()?;

    let Some(argument) = make_function_argument(lazy, stream, parent)? else {
      return stream.expected_here(line_dbg!("a function argument"));
    };

    stream.skip_whitespace_and_comments()?;

    arguments.push(argument);

    let Some((Token::Indent(0), _)) = stream.peek()? else {
      return stream.expected_here(line_dbg!("a newline (0 indent)"));
    };
    stream.seek();
  };

  let mut span = name.span;
  span.end = stream.here()?.start;

  Ok(Some(lang::function::FunctionHeader {
    name,
    ret_ty,
    arguments,
    span,
  }))
}

pub(super) fn make_function<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: lang::reference::ModuleReference,
) -> Result<Option<lang::reference::FunctionReference>, Error> {
  let Some(header) = make_function_header(lazy, stream, module)? else {
    return Ok(None);
  };

  let function = lazy.create_function(module, header);
  let body = lazy.rget(function).body;
  let mut non_return_last = None;

  let indenter = stream.indenter_here()?;

  loop {
    stream.skip_whitespace_and_comments()?;

    let Some(stmt) = expr::block::make_block_statement(
      lazy, stream, &indenter,
      module, function, body,
    )? else {
      return stream.expected_here(line_dbg!("a block statement"));
    };

    if stmt.non_return_last && stmt.expr.is_some() {
      non_return_last = stmt.expr;
    };

    if stmt.end {
      break;
    };
  };

  if let Some(&last) = body.rget_from(lazy).children.last() {
    let body_ref = function.get_body_mut(lazy);

    body_ref.returns_last = !non_return_last.is_some_and(
      |ExpressionReference(_, id)| id == last
    );

    let expr_reference = ExpressionReference(body, last);
    body_ref.out = lang::ty::Type::Reference(lang::reference::TypeReference::Expression(expr_reference));
  };

  function.rget_from_mut(lazy).span.end = stream.here()?.start;

  Ok(Some(function))
}
