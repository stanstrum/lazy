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

pub mod module {
  use crate::Lazy;
  use crate::lang::{ModuleReference, Reference, TypePartReference};
  use crate::lang::ty::Type;

  pub mod struc {
    pub type Struct = ::lang::module::Struct<crate::lazy::LazyStructures>;
  }

  pub mod import {
    pub type ImportGroup = ::lang::import::ImportGroup<crate::lazy::LazyStructures>;
    pub type ImportQualify = ::lang::import::ImportQualify<crate::lazy::LazyStructures>;
    pub type ImportPart = ::lang::import::ImportPart<crate::lazy::LazyStructures>;
    pub type Import = ::lang::import::Import<crate::lazy::LazyStructures>;
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub struct TokensId(pub usize);

  pub type ModulePath = ::lang::module::ModulePath<crate::lazy::LazyStructures>;
  pub type ModuleParent = ::lang::module::ModuleParent<crate::lazy::LazyStructures>;
  // pub type ModuleTransports = ::lang::module::ModuleTransports<crate::lazy::LazyStructures>;

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub struct TypePartId(pub usize);

  pub type Module = ::lang::module::Module<crate::lazy::LazyStructures>;
  pub type TypeAlias = ::lang::module::TypeAlias<crate::lazy::LazyStructures>;
  pub type Name = ::lang::module::Name<crate::lazy::LazyStructures>;

  impl ModuleReference {
    pub fn add_type_part(&self, part: Type, lazy: &mut Lazy) -> TypePartReference {
      let module_ref = self.rget_from_mut(lazy);

      let id = TypePartId(module_ref.type_parts.len());
      module_ref.type_parts.push(part);

      TypePartReference(*self, id)
    }
  }
}

pub mod function {
  pub type FunctionHeader = ::lang::function::FunctionHeader<crate::lazy::LazyStructures>;
  pub type Function = ::lang::function::Function<crate::lazy::LazyStructures>;
}
