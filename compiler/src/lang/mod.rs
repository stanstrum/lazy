pub mod reference;
pub mod module;
pub mod function;
pub mod expr;

mod get_span;

use std::path::PathBuf;

#[derive(Debug)]
pub enum LazyError {
  NotExist(PathBuf),
  Aster(crate::aster::Error),
}

pub mod ty {
  pub type QualifiedSearchSpace = ::lang::ty::QualifiedSearchSpace<crate::lazy::LazyStructures>;
  pub type Qualified = ::lang::ty::Qualified<crate::lazy::LazyStructures>;

  pub type Type = ::lang::ty::Type<crate::lazy::LazyStructures>;
}
