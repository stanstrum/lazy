use std::path::PathBuf;

use string_pool::PoolId;

use crate::Compiler;

#[derive(Debug, Clone, Copy)]
pub struct Name<C: Compiler> {
  pub id: PoolId,
  pub span: crate::span::Span<C>,
}

#[derive(Debug)]
pub struct ModulePath<C: Compiler> {
  pub path: PathBuf,
  pub tokens: C::TokensReference,
  pub module: C::ModuleReference,
}

#[derive(Debug)]
pub enum ModuleParent<C: Compiler> {
  Path(ModulePath<C>),
  Module(C::ModuleReference),
}
