mod module;

use std::path::PathBuf;
use std::ops::{Index, IndexMut};
use std::fs::File;

use crate::string_pool::StringPool;
pub use module::{
  Module,
  ModuleId,
  ModuleParent,
};

#[derive(Debug)]
pub struct Lazy<'a> {
  pub pool: &'a StringPool,
  modules: Vec<Module>,
}

impl<'a> Lazy<'a> {
  pub fn new(pool: &'a StringPool) -> Self {
    Self {
      pool,
      modules: vec![],
    }
  }

  pub fn add_file(&mut self, name: &str, path: PathBuf) -> ModuleId {
    let name = self.pool.insert(name);
    let id = self.modules.len();
    let parent = ModuleParent::Path { path, file: None };
    self.modules.push(Module::new(name, parent));
    ModuleId(id)
  }

  fn describe_module(&self, ModuleId(id): ModuleId) -> String {
    let module = self.modules.get(id).unwrap();
    let name = self.pool.get(module.name).collect::<String>();

    match &module.parent {
      ModuleParent::Path { path, .. } => {
        format!(
          "[{} = {}]",
          path.to_string_lossy().to_string(),
          name
        )
      },
      ModuleParent::Module(parent) => {
        let parent_desc = self.describe_module(*parent);
        format!("{parent_desc}::{name}")
      },
    }
  }

  pub fn open_file(&mut self, mut id: ModuleId) -> &File {
    // traverse parents until we get the root module with a
    // PathBuf
    loop {
      match &self[id].parent {
        ModuleParent::Path { .. } => break,
        ModuleParent::Module(next_id) => id = *next_id,
      };
    };

    // store and return the file handle
    let ModuleParent::Path { path, file } =
      &mut self[id].parent else { unreachable!(); };

    file.get_or_insert_with(|| {
      // TODO: error handling
      File::open(path).expect("failed to open source file")
    })
  }
}

impl<'a> Index<ModuleId> for Lazy<'a> {
  type Output = Module;

  fn index(&self, ModuleId(index): ModuleId) -> &Self::Output {
    self.modules.get(index).unwrap()
  }
}

impl<'a> IndexMut<ModuleId> for Lazy<'a> {
  fn index_mut(&mut self, ModuleId(index): ModuleId) -> &mut Self::Output {
    self.modules.get_mut(index).unwrap()
  }
}
