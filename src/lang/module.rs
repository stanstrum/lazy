use std::path::PathBuf;
use std::fs::File;

use crate::string_pool::PoolId;

#[derive(Debug, Clone, Copy)]
pub struct ModuleId(pub usize);

#[derive(Debug)]
pub enum ModuleParent {
  Path {
    path: PathBuf,
    file: Option<File>,
  },
  Module(ModuleId),
}

#[derive(Debug)]
pub struct Module {
  pub name: PoolId,
  pub modules: Vec<ModuleId>,
  pub parent: ModuleParent,
}

impl Module {
  pub fn new(name: PoolId, parent: ModuleParent) -> Self {
    Self {
      name,
      modules: vec![],
      parent,
    }
  }
}
