mod ty;
mod function;
mod expr;

use std::io::Read;

use crate::aster::pprint::Pretty;
use crate::lang;
use crate::tokenize::token::{Span, Token};
use crate::aster::Rereader;
use crate::aster::make::ty::make_type;

use super::Error;

#[macro_export]
macro_rules! line_dbg {
  ($str:expr) => {
    concat!("[\x1b[1;4m", file!(), ":", line!(), "\x1b[0;24m]: ", $str)
  };
}

impl<'pool, const N: usize, T: Read> Rereader<'pool, N, T> {
  fn done(&mut self) -> Result<bool, Error> {
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
    Ok(match self.peek()? {
      Some((_, span)) => span,
      _ => {
        let (_, last_span) = self.queue.iter().last().expect("Rereader::here peek for span");
        Span {
          module: last_span.module,
          start: last_span.end,
          end: last_span.end,
        }
      },
    })
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
  let mut last_mark = None;

  loop {
    // println!("{}", lazy[stream.id].print(lazy).collect::<Vec<String>>().join("\n"));

    stream.skip_whitespace_and_comments()?;

    if stream.done()? {
      break;
    };

    let current_mark = stream.mark();
    if last_mark == Some(current_mark) {
      return stream.expected_here(line_dbg!("a top-level structure"));
    };

    last_mark = Some(current_mark);

    if let Some(function) = function::make_function(lazy, stream)? {
      lazy.add_function(stream.id, function);
      continue;
    };
  };

  Ok(())
}
