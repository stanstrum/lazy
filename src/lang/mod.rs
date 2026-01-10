mod module;

use crate::string_pool::StringPool;
use module::{
  Module,
  ModuleId,
  ModuleParent,
};

#[derive(Debug)]
pub struct Lazy {
  pool: StringPool,
  modules: Vec<Module>,
}

impl Lazy {
  pub fn new() -> Self {
    Self {
      pool: StringPool::new(),
      modules: vec![],
    }
  }

  fn add_module(&mut self, module: Module) -> ModuleId {
    let id = self.modules.len();

    self.modules.push(module);

    ModuleId(id)
  }

  fn describe_module(&self, ModuleId(id): ModuleId) -> String {
    let module = self.modules.get(id).unwrap();
    let name = self.pool.get(module.name).collect::<String>();

    match &module.parent {
      ModuleParent::Path => {
        format!(
          "[{} = {}]",
          module.path.to_string_lossy().to_string(),
          name
        )
      },
      ModuleParent::Module(parent) => {
        let parent_desc = self.describe_module(*parent);
        format!("{parent_desc}::{name}")
      },
    }
  }
}

