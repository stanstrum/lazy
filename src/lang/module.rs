use std::path::PathBuf;

use crate::string_pool::PoolId;

#[derive(Debug, Clone, Copy)]
pub struct ModuleId(pub usize);

#[derive(Debug)]
pub enum ModuleParent {
  Path,
  Module(ModuleId),
}

#[derive(Debug)]
pub struct Module {
  pub path: PathBuf,
  pub name: PoolId,
  pub modules: Vec<ModuleId>,
  pub parent: ModuleParent,
}
