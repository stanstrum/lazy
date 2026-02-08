use crate::lang::Lazy;
use crate::lang::ty::Type;
use crate::lang::reference::Store;

use super::*;
pub trait TypeOf {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>, Error>;
}

impl<R: Copy> TypeOf for R
  where for<'a> Lazy<'a>: Store<R>,
        for<'a> <Lazy<'a> as Store<R>>::Out: TypeOf
{
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>, Error> {
    lazy.rget(*self).type_of(lazy)
  }
}
