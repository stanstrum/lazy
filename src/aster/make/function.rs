use crate::lang::reference::{ExpressionReference, Reference};
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

  let function = lazy.create_function(parent, header);
  let body = lazy.rget(function).body;
  let mut non_return_last = None;

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

    let expr = if let Some(expr) = expr::make_expr(lazy, stream, parent, function)? {
      Some(expr)
    } else if let Some((variable, expr)) = expr::variable::make_assignment(lazy, stream, parent, function)? {
      let function_ref = lazy.rget(function);
      let body_ref = lazy.rget(body);

      let variable_names = body_ref.variables.iter().map(|x: &lang::expr::Variable| &x.name);
      let argument_names = function_ref.header.arguments.iter().map(|x| &x.name);

      let conflict = argument_names.chain(variable_names)
        .find(|prior: &&lang::module::Name| prior.id == variable.name.id);

      if let Some(conflict) = conflict {
        print_message(lazy, PrintableMessage {
          level: Level::Warn,
          force: false,
          description: "conflicting name will be shadowed".into(),
          contents: MessageContents::WithinSource {
            range: function_ref.span,
            sections: vec![
              MessageSection {
                text: "first used here".into(),
                span: conflict.span,
              },
              MessageSection {
                text: "shadowed here".into(),
                span: variable.name.span,
              },
            ],
          },
        });
      };

      body.rget_from_mut(lazy).variables.push(variable);
      expr
    } else {
      return stream.expected_here(line_dbg!("an expression"));
    };

    let id = expr.map(|expr| {
      lazy.rget_mut(function).add_expr_to_block(expr, body)
    });

    stream.skip_whitespace_and_comments()?;

    if let Some((Token::Operator(Operator::Semicolon), _)) = stream.peek()? {
      stream.seek();
      stream.skip_whitespace_and_comments()?;

      if id.is_some() {
        non_return_last = id;
      };
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

  if let Some(ExpressionReference(_, last_id)) = function.last_expr(lazy) {
    function.get_body_mut(lazy).returns_last = !non_return_last.is_some_and(
      |id| id == last_id
    );
  };

  function.rget_from_mut(lazy).span.end = stream.here()?.start;

  Ok(Some(function))
}
