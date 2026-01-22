use std::io::Read;

use crate::lang;
use crate::aster::Rereader;
use crate::lang::module::ModuleId;
use crate::tokenize::token::{Operator, Token};

use super::Error;

pub(super) fn make_type<'pool, const N: usize, T: Read>(
  _lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: ModuleId,
) -> Result<Option<lang::ty::Type>, Error> {
  let ret_mark = stream.mark();

  if let Some((Token::Identifier(first), first_span)) = stream.peek()? {
    let first_name = lang::module::Name { id: first, span: first_span };
    let mut parts = vec![first_name];
    stream.seek();

    let ret_mark = stream.mark();

    stream.skip_whitespace_and_comments()?;

    if let Some((Token::Operator(Operator::DoubleColon), _)) = stream.ok_next()? {
      todo!("qualified deep read")
    } else {
      stream.take_mark(ret_mark);

      Ok(Some(lang::ty::Type::Unresolved {
        module,
        qualified: lang::ty::Qualified {
          parts,
          span: first_span,
        },
      }))
    }
  } else {
    stream.take_mark(ret_mark);
    Ok(None)
  }
}
