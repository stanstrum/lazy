pub mod module;
pub mod function;
pub mod ty;
pub mod expr;

use std::path::{Path, PathBuf};
use std::ops::{Index, IndexMut};

use crate::lang::function::Function;
use crate::lang::module::{FunctionId, Module, ModuleId, ModuleParent};
use crate::string_pool::StringPool;

#[derive(Debug)]
pub struct Lazy<'a> {
  pub pool: &'a StringPool,
  modules: Vec<Module>,
  functions: Vec<Function>,
}

impl<'a> Lazy<'a> {
  pub fn new(pool: &'a StringPool) -> Self {
    Self {
      pool,
      modules: vec![],
      functions: vec![],
    }
  }

  pub fn add_file(&mut self, name: &str, path: PathBuf) -> ModuleId {
    let name = self.pool.insert(name);
    let id = ModuleId(self.modules.len());
    let parent = ModuleParent::Path { path, opened: false };
    self.modules.push(Module::new(name, parent));
    id
  }

  pub fn add_function(&mut self, module_id: ModuleId, function: Function) -> FunctionId {
    let function_id = FunctionId(self.functions.len());

    self.functions.push(function);
    self[module_id].functions.push(function_id);

    function_id
  }

  fn describe_module(&self, ModuleId(id): ModuleId) -> String {
    let module = self.modules.get(id).unwrap();
    let name = self.pool.get(module.name).collect::<String>();

    match &module.parent {
      ModuleParent::Path { path, .. } => {
        format!(
          "[{} = {}]",
          path.to_string_lossy(),
          name
        )
      },
      ModuleParent::Module(parent) => {
        let parent_desc = self.describe_module(*parent);
        format!("{parent_desc}::{name}")
      },
    }
  }

  pub fn get_path(&mut self, mut id: ModuleId) -> &Path {
    // traverse parents until we get the root module with a
    // PathBuf
    loop {
      match &self[id].parent {
        ModuleParent::Path { .. } => break,
        ModuleParent::Module(next_id) => id = *next_id,
      };
    };

    // store and mark the file handle as read
    let ModuleParent::Path { path, opened } =
      &mut self[id].parent else { unreachable!(); };

    assert!(!*opened);
    *opened = true;

    path.as_path()
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

impl<'a> Index<FunctionId> for Lazy<'a> {
  type Output = Function;

  fn index(&self, FunctionId(index): FunctionId) -> &Self::Output {
    self.functions.get(index).unwrap()
  }
}

impl<'a> IndexMut<FunctionId> for Lazy<'a> {
  fn index_mut(&mut self, FunctionId(index): FunctionId) -> &mut Self::Output {
    self.functions.get_mut(index).unwrap()
  }
}
