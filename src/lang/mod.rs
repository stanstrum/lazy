pub mod reference;
pub mod module;
pub mod function;
pub mod ty;
pub mod expr;
pub mod span;

use std::path::PathBuf;

use crate::print_message;
use crate::settings::Settings;
use crate::settings::format::format_argument;
use crate::string_pool::StringPool;

use crate::lang::function::{Function, FunctionHeader};
use crate::lang::module::{Module, ModuleParent, ModulePath, TokensId};
use crate::lang::reference::{FunctionReference, ModuleReference, Store};
use crate::tokenize::token;

#[derive(Debug)]
pub struct Lazy<'a> {
  pub pool: &'a StringPool,
  pub settings: Settings,
  modules: Vec<Module>,
  functions: Vec<Function>,
  tokens: Vec<Vec<token::TokenSpan>>,
}

impl<'a> Lazy<'a> {
  pub fn new(pool: &'a StringPool, settings: Settings) -> Self {
    let lazy = Self {
      pool,
      settings,
      modules: vec![],
      functions: vec![],
      tokens: vec![],
    };

    let argv = lazy.settings.argv.iter()
      .map(|arg| format_argument(&arg))
      .collect::<Vec<_>>()
      .join(" ");

    print_message!(&lazy, {
      level: Level::Debug,
      force: false,
      description: argv,
      contents: MessageContents::None,
    });

    lazy
  }

  pub fn add_file(&mut self, name: &str, path: PathBuf) -> ModuleReference {
    let name = self.pool.insert(name);
    let module = ModuleReference(self.modules.len());
    let tokens = TokensId(self.tokens.len());
    let parent = ModuleParent::Path(ModulePath { path, tokens, module });
    self.tokens.push(vec![]);
    self.modules.push(Module::new(name, parent));
    module
  }

  pub fn create_function(&mut self, module: ModuleReference, header: FunctionHeader) -> FunctionReference {
    let function_id = FunctionReference(self.functions.len());
    let function = Function::new(function_id, module, header);

    self.functions.push(function);
    self.rget_mut(module).functions.push(function_id);

    function_id
  }

  pub fn describe_module(&self, ModuleReference(id): ModuleReference) -> String {
    let module = self.modules.get(id).unwrap();
    let name = self.pool.get(module.name).collect::<String>();

    match &module.parent {
      ModuleParent::Path(ModulePath { /* path, */ .. }) => {
        // let mut path = path.as_path();

        // if
        //   let Some(parent) = self.settings.input_path.parent() &&
        //   let Ok(stripped) = path.strip_prefix(parent)
        // {
        //   path = stripped;
        // };

        // format!(
        //   "[{}:{}]",
        //   name,
        //   path.to_string_lossy(),
        // )
        name
      },
      ModuleParent::Module(parent) => {
        let parent_desc = self.describe_module(*parent);
        format!("{parent_desc}::{name}")
      },
    }
  }

  pub fn get_root_module(&self, mut module: ModuleReference) -> ModuleReference {
    // traverse parents until we get the root module with a
    // PathBuf
    loop {
      match &self.rget(module).parent {
        ModuleParent::Path { .. } => break,
        &ModuleParent::Module(next_id) => module = next_id,
      };
    };

    // store and mark the file handle as read
    let ModuleParent::Path(_) = &self.rget(module).parent else {
      unreachable!();
    };

    module
  }

  pub fn get_path(&self, module: ModuleReference) -> &ModulePath {
    let root = self.get_root_module(module);
    let root = self.rget(root);

    // store and mark the file handle as read
    let ModuleParent::Path(path) = &root.parent else {
      unreachable!();
    };

    path
  }
}
