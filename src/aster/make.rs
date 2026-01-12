use std::io::Read;

use crate::lang;
use crate::tokenize::token::{Span, Token, Operator};
use crate::aster::Rereader;

use super::Error;

impl<'pool, const N: usize, T: Read> Rereader<'pool, N, T> {
  fn done(&mut self) -> Result<bool, Error> {
    Ok(self.peek()?.is_none())
  }

  fn skip_whitespace_and_comments(&mut self) -> Result<bool, Error> {
    let mut did_skip = false;

    loop {
      let Some((Token::Whitespace, _)) = self.peek()? else {
        break;
      };

      self.seek();
      did_skip = true;
    };

    Ok(did_skip)
  }

  fn here(&mut self) -> Result<Span, Error> {
    let (_, span) = self.peek()?.expect("Rereader::here peek for span");
    Ok(span)
  }

  fn expected_here<V>(&mut self, what: &'static str) -> Result<V, Error> {
    Err(Error::Expected {
      what,
      at: self.here()?,
    })
  }
}

fn make_type<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
) -> Result<Option<lang::ty::Type>, Error> {
  todo!("type")
}

fn make_function_header<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
) -> Result<Option<lang::function::FunctionHeader>, Error> {
  let Some((Token::Identifier(ident), span)) = stream.ok_next()? else {
    return Ok(None);
  };

  stream.skip_whitespace_and_comments()?;

  let ret_ty = {
    if let Some((Token::Operator(Operator::RightArrow), _)) = stream.peek()? {
      stream.seek();
      stream.skip_whitespace_and_comments()?;

      let Some(ret_ty) = make_type(lazy, stream)? else {
        return stream.expected_here("a return type");
      };

      Some(ret_ty)
    } else {
      None
    }
  };

  stream.skip_whitespace_and_comments()?;

  let Some((Token::Indent(1..), _)) = stream.ok_next()? else {
    return stream.expected_here("an indentation");
  };

  todo!("make_function_header");

  Ok(Some(lang::function::FunctionHeader {
    name: ident,
    ret_ty,
    arguments: todo!(),
    span,
  }))
}

fn make_function<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
) -> Result<Option<lang::function::Function>, Error> {
  let ret_mark = stream.mark();

  let Some((_, start)) = stream.peek()? else {
    stream.take_mark(ret_mark);
    return Ok(None);
  };

  let Some(header) = make_function_header(lazy, stream)? else {
    stream.take_mark(ret_mark);
    return Ok(None);
  };

  let Some((_, end)) = stream.peek()? else {
    panic!("no end span");
  };

  Ok(Some(lang::function::Function {
    header,
    span: Span::from_pair(stream.id, start, end),
  }))
}

pub(super) fn make<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>
) -> Result<(), Error> {
  loop {
    stream.skip_whitespace_and_comments()?;

    if stream.done()? {
      break;
    };

    if let Some(function) = make_function(lazy, stream)? {
      todo!("{function:?}");
    };

    todo!("make_function loop")
  };

  todo!("make")
}
