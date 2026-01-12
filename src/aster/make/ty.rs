use std::io::Read;

use crate::lang;
use crate::aster::Rereader;

use super::Error;

pub(super) fn make_type<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
) -> Result<Option<lang::ty::Type>, Error> {
  todo!("type")
}
