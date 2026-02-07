use crate::lang::span::GetSpan;
use crate::line_dbg;

use crate::tokenize::token::Operator;

use std::cmp::Ordering;
use super::*;

fn make_function_argument<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  parent: lang::reference::ModuleReference,
) -> Result<Option<lang::function::FunctionArgument>, Error> {
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

  Ok(Some(lang::function::FunctionArgument {
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
  parent: lang::reference::ModuleReference,
) -> Result<Option<lang::reference::FunctionReference>, Error> {
  let Some(header) = make_function_header(lazy, stream, parent)? else {
    return Ok(None);
  };

  let function = lang::function::Function::new(parent, header);
  let mut non_return_last = None;

  let function_id = lazy.add_function(parent, function);

  loop {
    stream.skip_whitespace_and_comments()?;

    if let Some((Token::Indent(indent), _)) = stream.peek()? {
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
            what: line_dbg!("a newline (0 or negative indent)"),
            at: stream.here()?,
          });
        },
      };
    };

    let Some(expr) = expr::make_expr(lazy, stream, function_id)? else {
      return stream.expected_here(line_dbg!("an expression"));
    };
    let id = todo!() /* lazy.rget_mut(function_id).add_expr_to_block(expr, body) */;

    stream.skip_whitespace_and_comments()?;

    if let Some((Token::Operator(Operator::Semicolon), _)) = stream.peek()? {
      stream.seek();
      stream.skip_whitespace_and_comments()?;

      non_return_last = Some(id);
    };

    let Some((Token::Indent(indent), _)) = stream.peek()? else {
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
          what: line_dbg!("a newline (0 or negative indent)"),
          at: stream.here()?,
        });
      },
    };
  };

  todo!();
  // lazy[function_id][body].returns_last = !non_return_last.is_some_and(
  //   |id| id == *lazy[function_id][body].children.last().unwrap()
  // );

  // lazy[function_id].span.end = stream.here()?.start;

  Ok(Some(function_id))
}
