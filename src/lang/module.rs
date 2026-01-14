use std::path::PathBuf;

use crate::string_pool::PoolId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModuleId(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct FunctionId(pub usize);

#[derive(Debug)]
pub enum ModuleParent {
  Path {
    path: PathBuf,
    opened: bool,
  },
  Module(ModuleId),
}

#[derive(Debug)]
pub struct Module {
  pub name: PoolId,
  pub modules: Vec<ModuleId>,
  pub functions: Vec<FunctionId>,
  pub parent: ModuleParent,
}

impl Module {
  pub fn new(name: PoolId, parent: ModuleParent) -> Self {
    Self {
      name,
      modules: vec![],
      functions: vec![],
      parent,
    }
  }
}
