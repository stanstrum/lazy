use crate::lang::module::ModuleId;

use super::*;

#[derive(Debug)]
pub enum Structure {
  Function(lang::function::Function),
}

pub(super) fn make_structure<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  parent: ModuleId,
) -> Result<Option<Structure>, Error> {
  Ok(if let Some(function) = function::make_function(lazy, stream, parent)? {
    Some(Structure::Function(function))
  } else {
    None
  })
}
