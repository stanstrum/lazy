pub mod reference;
pub mod module;
pub mod function;

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

pub mod expr {
  pub mod operator {
    pub use ::lang::expr::operator::*;
    pub type UnarySuffixOperator = ::lang::expr::operator::UnarySuffixOperator<crate::lazy::LazyStructures>;
  }

  pub type Variable = ::lang::expr::Variable<crate::lazy::LazyStructures>;
  pub type BlockExpression = ::lang::expr::BlockExpression<crate::lazy::LazyStructures>;
  pub type LiteralKind = ::lang::expr::LiteralKind;
  pub type Expression = ::lang::expr::Expression<crate::lazy::LazyStructures>;
}
