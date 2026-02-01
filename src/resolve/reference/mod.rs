mod impls;
pub use impls::*;

use crate::lang;

pub trait Store<'a, R: Reference<'a>> {
  fn rget(&'a self, reference: &R) -> &'a R::Out;
  fn rget_mut(&'a mut self, reference: &R) -> &'a mut R::Out;
}

impl<'a, R: Reference<'a>> Store<'a, R> for R::Parent<'_> {
  fn rget(&'a self, reference: &R) -> &'a R::Out {
    reference.rget_from(self)
  }

  fn rget_mut(&'a mut self, reference: &R) -> &'a mut R::Out {
    reference.rget_from_mut(self)
  }
}

pub trait Reference<'a>: std::fmt::Debug {
  type Parent<'b>;
  type Out;

  fn rget_from(&self, parent: &'a Self::Parent<'_>) -> &'a Self::Out;
  fn rget_from_mut(&self, parent: &'a mut Self::Parent<'_>) -> &'a mut Self::Out;
}
