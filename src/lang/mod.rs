pub mod module;
pub mod function;
pub mod ty;
pub mod expr;
pub mod span;

use std::path::PathBuf;
use std::ops::{Index, IndexMut};

use crate::lang::function::Function;
use crate::lang::module::{FunctionId, Module, ModuleId, ModuleParent, ModulePath, TokensId};
use crate::string_pool::StringPool;
use crate::tokenize::token;

#[derive(Debug)]
pub struct Lazy<'a> {
  pub pool: &'a StringPool,
  modules: Vec<Module>,
  functions: Vec<Function>,
  tokens: Vec<Vec<token::TokenSpan>>,
}

impl<'a> Lazy<'a> {
  pub fn new(pool: &'a StringPool) -> Self {
    Self {
      pool,
      modules: vec![],
      functions: vec![],
      tokens: vec![],
    }
  }

  pub fn add_file(&mut self, name: &str, path: PathBuf) -> ModuleId {
    let name = self.pool.insert(name);
    let module = ModuleId(self.modules.len());
    let tokens = TokensId(self.tokens.len());
    let parent = ModuleParent::Path(ModulePath { path, tokens });
    self.tokens.push(vec![]);
    self.modules.push(Module::new(name, parent));
    module
  }

  pub fn add_function(&mut self, module_id: ModuleId, function: Function) -> FunctionId {
    let function_id = FunctionId(self.functions.len());

    self.functions.push(function);
    self[module_id].functions.push(function_id);

    function_id
  }

  pub fn describe_module(&self, ModuleId(id): ModuleId) -> String {
    let module = self.modules.get(id).unwrap();
    let name = self.pool.get(module.name).collect::<String>();

    match &module.parent {
      ModuleParent::Path(ModulePath { path, .. }) => {
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

  pub fn get_path(&self, mut id: ModuleId) -> &ModulePath {
    // traverse parents until we get the root module with a
    // PathBuf
    loop {
      match &self[id].parent {
        ModuleParent::Path { .. } => break,
        ModuleParent::Module(next_id) => id = *next_id,
      };
    };

    // store and mark the file handle as read
    let ModuleParent::Path(path) = &self[id].parent else {
      unreachable!();
    };

    path
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

impl<'a> Index<TokensId> for Lazy<'a> {
  type Output = Vec<token::TokenSpan>;

  fn index(&self, TokensId(index): TokensId) -> &Self::Output {
    self.tokens.get(index).unwrap()
  }
}

impl<'a> IndexMut<TokensId> for Lazy<'a> {
  fn index_mut(&mut self, TokensId(index): TokensId) -> &mut Self::Output {
    self.tokens.get_mut(index).unwrap()
  }
}
