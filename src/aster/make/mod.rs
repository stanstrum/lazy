mod ty;
mod function;

use std::io::Read;

use crate::lang;
use crate::tokenize::token::{Span, Token};
use crate::aster::Rereader;
use crate::aster::make::ty::make_type;

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

pub(super) fn make<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>
) -> Result<(), Error> {
  loop {
    stream.skip_whitespace_and_comments()?;

    if stream.done()? {
      break;
    };

    if let Some(function) = function::make_function(lazy, stream)? {
      todo!("{function:?}");
    };

    todo!("make_function loop")
  };

  todo!("make")
}
