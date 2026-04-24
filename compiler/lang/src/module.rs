use std::path::PathBuf;

use string_pool::PoolId;

#[derive(Debug, Clone, Copy)]
pub struct Name<M> {
  pub id: PoolId,
  pub span: crate::span::ModuleSpan<M>,
}

#[derive(Debug)]
pub struct ModulePath<I, M> {
  pub path: PathBuf,
  pub tokens: I,
  pub module: M,
}

#[derive(Debug)]
pub enum ModuleParent<I, M> {
  Path(ModulePath<I, M>),
  Module(M),
}

