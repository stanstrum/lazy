pub mod reference;
pub mod module;
pub mod function;
pub mod ty;
pub mod expr;
pub mod span;

use std::path::PathBuf;

#[derive(Debug)]
pub enum LazyError {
  NotExist(PathBuf),
  Aster(crate::aster::Error),
}

impl From<crate::aster::Error> for LazyError {
  fn from(value: crate::aster::Error) -> Self {
    Self::Aster(value)
  }
}
