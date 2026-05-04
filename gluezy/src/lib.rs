mod reference;
pub mod format;

use std::path::{Path, PathBuf};

use string_pool::StringPool;
use lazy_macros::{line_dbg, print_message};
use log::Level;
use lang::error::LazyError;
use lang::token::Tokens;
use lang::reference::Store;
use lang::module::{Module, ModuleParent, ModulePath};
use lang::function::{Function, FunctionHeader};
use lang::{Compiler, CompilerPoolStore};

pub use reference::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LazyStructures;

#[derive(Debug)]
pub struct Settings {
  pub executable: String,
  pub input_path: PathBuf,
  pub output_path: PathBuf,
  pub log_level: Level,
  pub argv: Vec<String>,
}

#[derive(Debug)]
pub struct Lazy<'pool, C: Compiler = LazyStructures> { pub pool: &'pool StringPool,
  pub pool_keys: lang::keys::PoolKeys,
  pub settings: Settings,
  pub std: Option<C::ModuleReference>,
  pub(crate) modules: Vec<Module<C>>,
  pub(crate) functions: Vec<Function<C>>,
  pub(crate) tokens: Vec<Tokens<C>>,
}

impl<'pool> Lazy<'pool> {
  pub fn new(pool: &'pool StringPool, settings: Settings) -> Self {
    let lazy = Self {
      pool,
      settings,
      std: None,
      pool_keys: lang::keys::PoolKeys::init(pool),
      modules: vec![],
      functions: vec![],
      tokens: vec![],
    };

    let argv = lazy.settings.argv.iter()
      .map(|arg| format::format_argument(arg))
      .collect::<Vec<_>>()
      .join(" ");

    print_message!(&lazy, {
      level: Debug,
      force: false,
      description: argv,
      contents: MessageContents::None::<LazyStructures>,
    });

    lazy
  }
}

// TODO: find a nicer spot for this stuff
impl<'pool> lang::CompilerPoolStore<'pool, LazyStructures> for Lazy<'pool> {
  type Error = LazyError<LazyStructures>;

  fn pool(&self) -> &'pool StringPool {
    self.pool
  }

  fn pool_keys(&self) -> &lang::keys::PoolKeys {
    &self.pool_keys
  }

  fn unwrap_std(&self) -> ModuleReference {
    self.std.unwrap()
  }

  fn get_std(&mut self) -> Result<<LazyStructures as Compiler>::ModuleReference, LazyError<LazyStructures>> {
    if let Some(std) = self.std {
      return Ok(std);
    };

    print_message!(self, {
      level: Stub,
      force: false,
      description: line_dbg!("@std can only be imported from cwd").into(),
      contents: MessageContents::None::<LazyStructures>,
    });

    let std_reference = match CompilerPoolStore::add_file(self, "@std", "std".into(), None) {
      Ok(x) => x,
      Err(err) => todo!("{err:?}"),
    };

    Ok(*self.std.insert(std_reference))
  }

  fn add_file(&mut self, name: &str, mut path: PathBuf, relative_to: Option<&Path>) -> Result<ModuleReference, LazyError<LazyStructures>> {
    // Make sure relative_to is absolute
    if let Some(relative_to) = &relative_to {
      assert!(relative_to.is_absolute(), "relative_to must be an absolute path");
    };

    // Prefix path with relative_to or cwd
    if path.is_relative() {
      let cwd; // SPONGE: there's certainly a better way to do this
      let relative_to = if let Some(relative_to) = relative_to {
        relative_to
      } else {
        cwd = std::env::current_dir()
          .expect("cwd to return current directory");

        cwd.as_path()
      };

      path = relative_to.join(path);
    };

    // If it's a directory, we'll take the index.zy module from it, if it
    // exists
    if path.is_dir() {
      path.push("index");
    };

    if
      // If the path, as it stands, is not a file,
      !path.is_file() &&
      // And it _does_ have a file name component,
      let Some(fname) = path.file_name() &&
      // And said component does not end with our extension,
      !fname.to_string_lossy().ends_with(".zy")
    {
      // Get our own copy
      let mut fname = fname.to_owned();

      // Then tack on that extension and try again.  When we try again, the
      // above conditions should prevent this happening more than once
      fname.push(".zy");

      // Apply it to the filename
      path.set_file_name(fname);
    };

    print_message!(self, {
      level: Debug,
      force: false,
      description: format!("Adding module {name:?} from {path:?}"),
      contents: MessageContents::None::<LazyStructures>,
    });

    // Check if our file is actually there
    if !path.is_file() {
      return Err(LazyError::NotExist(path));
    };

    let parent = |tokens, module| ModuleParent::Path(ModulePath { path, tokens, module });
    let module = self.create_module(name, parent);

    // Tokenize, asterize (parse AST)
    if let Err(err) = aster::asterize::<LazyStructures>(self, module) {
      return Err(LazyError::Aster(err));
    };

    Ok(module)
  }

  fn create_function(&mut self, module: ModuleReference, header: impl FnOnce(FunctionReference) -> FunctionHeader<LazyStructures>) -> FunctionReference {
    let function_reference = FunctionReference(self.functions.len());
    let header = header(function_reference);
    let function = Function::new(function_reference, module, header);

    self.functions.push(function);
    self.rget_mut(module).functions.push(function_reference);

    function_reference
  }

  fn create_module(&mut self, name: &str, parent: impl FnOnce(TokensId, ModuleReference) -> ModuleParent<LazyStructures>) -> ModuleReference {
    // Make the references for this file
    let module = ModuleReference(self.modules.len());
    let tokens = TokensId(self.tokens.len());

    // Initialize the module struct
    let name = self.pool.insert(name);
    let to_insert = Module::new(name, parent(tokens, module));

    // Store the module's entries
    self.modules.push(to_insert);
    self.tokens.push(vec![]);

    module
  }
}

impl lang::Compiler for LazyStructures {
  type Store<'pool> = Lazy<'pool>;

  type ModuleReference = ModuleReference;
  type FunctionReference = FunctionReference;

  type TokensReference = TokensId;
}
