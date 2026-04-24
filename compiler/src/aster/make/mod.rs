mod ty;
mod function;
mod expr;
mod structure;

use std::io::Read;

use crate::lang;
use crate::lang::{Reference, Store, Span, Token, TokenSpan};
use crate::aster::Rereader;

use super::Error;

#[macro_export]
macro_rules! line_dbg {
  () => {
    line_dbg!("")
  };

  ($str:expr) => {
    concat!("[\x1b[1;4m", file!(), ":", line!(), "\x1b[0;24m]: ", $str)
  };
}

#[derive(Debug)]
struct Indenter(pub usize);

impl Indenter {
  fn peek<'pool, const N: usize, T: Read>(
    &self, stream: &mut Rereader<'pool, N, T>
  ) -> Result<Option<TokenSpan>, Error> {
    let Some(peek @ (_, span)) = stream.peek()? else {
      return Ok(None);
    };

    if span.start.indentation < self.0 {
      return Ok(None);
    };

    Ok(Some(peek))
  }

  // fn next<'pool, const N: usize, T: Read>(
  //   &self, stream: &mut Rereader<'pool, N, T>
  // ) -> Result<Option<TokenSpan>, Error> {
  //   let peek = self.peek(stream)?;

  //   if peek.is_some() {
  //     stream.seek();
  //   };

  //   Ok(peek)
  // }
}

impl<'pool, const N: usize, T: Read> Rereader<'pool, N, T> {
  fn indenter_here(&mut self) -> Result<Indenter, Error> {
    let (_, peek_span) = self.peek()?.expect("there to be another token");
    Ok(Indenter(peek_span.start.indentation))
  }

  fn is_done(&mut self) -> Result<bool, Error> {
    Ok(self.peek()?.is_none())
  }

  fn skip_whitespace_and_comments(&mut self) -> Result<bool, Error> {
    let mut did_skip = false;

    loop {
      let Some((Token::Whitespace | Token::Comment(_), _)) = self.peek()? else {
        break;
      };

      self.seek();
      did_skip = true;
    };

    Ok(did_skip)
  }

  fn here(&mut self) -> Result<Span, Error> {
    if let Some((_, span)) = self.peek()? {
      return Ok(span);
    };

    let (_, last_span) = self.queue.iter()
      .last()
      .expect("Rereader::here peek for span");

    Ok(Span {
      module: last_span.module,
      start: last_span.end,
      end: last_span.end,
    })
  }

  fn expected_here<V>(&mut self, what: &'static str) -> Result<V, Error> {
    Err(Error::Expected {
      what,
      at: self.here()?,
    })
  }
}

fn make_name<'pool, const N: usize, T: Read>(
  stream: &mut Rereader<'pool, N, T>
) -> Result<Option<lang::module::Name>, Error> {
  let Some((Token::Identifier(id), span)) = stream.peek()? else {
    return Ok(None);
  };
  stream.seek();

  Ok(Some(lang::module::Name { id, span }))
}

pub(super) fn make<'pool, const N: usize, T: Read>(
  lazy: &mut crate::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>
) -> Result<(), Error> {
  loop {
    stream.skip_whitespace_and_comments()?;

    if stream.is_done()? {
      break;
    };

    if let Some((Token::Indent(0), _)) = stream.peek()? {
      stream.seek();
      continue;
    };

    let Some(_) = structure::make_structure(lazy, stream.module, stream)? else {
      return stream.expected_here(line_dbg!("a top-level structure"));
    };
  };

  Ok(())
}
