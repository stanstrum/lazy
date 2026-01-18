use crate::aster::pprint::PrettyFunction;
use crate::line_dbg;
use std::io::Read;

use crate::lang;
use crate::tokenize::token::{Operator, Span, Token};
use crate::aster::make::{expr, make_type};
use crate::aster::Rereader;

use super::Error;

fn make_function_header<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
) -> Result<Option<lang::function::FunctionHeader>, Error> {
  let ret_mark = stream.mark();

  let Some((Token::Identifier(name), name_span)) = stream.ok_next()? else {
    stream.take_mark(ret_mark);
    return Ok(None);
  };

  stream.skip_whitespace_and_comments()?;

  let ret_ty = {
    if let Some((Token::Operator(Operator::RightArrow), _)) = stream.peek()? {
      stream.seek();
      stream.skip_whitespace_and_comments()?;

      let Some(ret_ty) = make_type(lazy, stream)? else {
        stream.take_mark(ret_mark);
        return stream.expected_here("a return type");
      };

      Some(ret_ty)
    } else {
      None
    }
  };

  stream.skip_whitespace_and_comments()?;

  let Some((Token::Indent(0..), _)) = stream.ok_next()? else {
    return stream.expected_here(line_dbg!("an indentation"));
  };

  let mut arguments = vec![];
  loop {
    if let Some((Token::Indent(indent), indent_span)) = stream.peek()? {
      stream.seek();

      match indent {
        1.. if arguments.is_empty() => break,
        0 => {},
        _other => {
          stream.take_mark(ret_mark);
          return Err(Error::Invalid {
            what: line_dbg!("indentation (expected 0)"),
            at: indent_span,
          });
        },
      };

      break;
    };

    let arg_ty_span = stream.here()?;
    let Some(arg_ty) = make_type(lazy, stream)? else {
      stream.take_mark(ret_mark);
      return stream.expected_here("a type");
    };

    if !stream.skip_whitespace_and_comments()? {
      stream.take_mark(ret_mark);
      return stream.expected_here("whitespace");
    };

    let Some((arg_name_token, arg_name_span)) = stream.ok_next()? else {
      stream.take_mark(ret_mark);
      return stream.expected_here("a token");
    };

    let Token::Identifier(arg_name) = arg_name_token else {
      stream.take_mark(ret_mark);
      return Err(Error::Expected {
        what: "an identifier",
        at: arg_name_span,
      });
    };

    let span = Span::from_pair(stream.id, arg_ty_span, arg_name_span);
    arguments.push(lang::function::FunctionArgument {
      name: arg_name,
      ty: arg_ty,
      span,
    });

    let Some((Token::Indent(indent), indent_span)) = stream.ok_next()? else {
      stream.take_mark(ret_mark);
      return stream.expected_here(line_dbg!("an indentation"));
    };

    if indent != 0 {
      return Err(Error::Invalid {
        what: line_dbg!("indentation (expected 0)"),
        at: indent_span,
      });
    };
  };

  let span = Span::from_pair(stream.id, name_span, stream.here()?);
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
) -> Result<Option<lang::function::Function>, Error> {
  let Some(header) = make_function_header(lazy, stream)? else {
    return Ok(None);
  };

  let header_span = header.span;

  let (mut function, body_id) = lang::function::Function::new(header);

  let (start_indent, mut curr_indent) = (
    header_span.start.indentation as isize,
    header_span.end.indentation as isize,
  );
  loop {
    if start_indent == curr_indent {
      break;
    };

    if let Some(expr) = dbg!(expr::make_expr(lazy, stream, &mut function)?) {
      println!("{}", expr.print_with(&function, lazy).collect::<Vec<_>>().join("\n"));
      function[body_id].children.push(expr);

      stream.skip_whitespace_and_comments()?;

      let Some((Token::Indent(indent), _)) = stream.peek()? else {
        // stream.take_mark(ret_here)
        return stream.expected_here(line_dbg!("a newline"));
      };

      curr_indent += indent;

      stream.seek();
      continue;
    };

    todo!("didn't make expr: {:?}", stream.peek()?)
  };

  Ok(Some(function))
}
