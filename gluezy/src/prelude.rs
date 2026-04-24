use std::path::PathBuf;

pub use crate::LazyError;

pub use ::lang::token::Token;
pub type TokenSpan = ::lang::token::TokenSpan<crate::LazyStructures>;
pub use ::lang::span::Position;
pub type Span = ::lang::span::Span<crate::LazyStructures>;

pub mod ty {
  pub type QualifiedSearchSpace = ::lang::ty::QualifiedSearchSpace<crate::LazyStructures>;
  pub type Qualified = ::lang::ty::Qualified<crate::LazyStructures>;

  pub type Type = ::lang::ty::Type<crate::LazyStructures>;
}

pub mod expr {
  pub mod operator {
    pub use ::lang::expr::operator::*;
    pub type UnarySuffixOperator = ::lang::expr::operator::UnarySuffixOperator<crate::LazyStructures>;
  }

  pub type Variable = ::lang::expr::Variable<crate::LazyStructures>;
  pub type BlockExpression = ::lang::expr::BlockExpression<crate::LazyStructures>;
  pub type LiteralKind = ::lang::expr::LiteralKind;
  pub type Expression = ::lang::expr::Expression<crate::LazyStructures>;
}

pub mod module {
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  pub struct ModuleReference(pub usize);

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  pub struct FunctionReference(pub usize);

  pub type TypePartReference = ::lang::reference::TypePartReference<crate::LazyStructures>;

  pub mod struc {
    pub type Struct = ::lang::module::Struct<crate::LazyStructures>;
  }

  pub mod import {
    pub type ImportGroup = ::lang::import::ImportGroup<crate::LazyStructures>;
    pub type ImportQualify = ::lang::import::ImportQualify<crate::LazyStructures>;
    pub type ImportPart = ::lang::import::ImportPart<crate::LazyStructures>;
    pub type Import = ::lang::import::Import<crate::LazyStructures>;
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub struct TokensId(pub usize);

  pub type ModulePath = ::lang::module::ModulePath<crate::LazyStructures>;
  pub type ModuleParent = ::lang::module::ModuleParent<crate::LazyStructures>;
  // pub type ModuleTransports = ::lang::module::ModuleTransports<crate::LazyStructures>;

  pub type Module = ::lang::module::Module<crate::LazyStructures>;
  pub type TypeAlias = ::lang::module::TypeAlias<crate::LazyStructures>;
  pub type Name = ::lang::module::Name<crate::LazyStructures>;
}

pub mod function {
  pub type FunctionHeader = ::lang::function::FunctionHeader<crate::LazyStructures>;
  pub type Function = ::lang::function::Function<crate::LazyStructures>;
}
