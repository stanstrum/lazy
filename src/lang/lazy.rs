use super::*;

#[derive(Debug)]
pub struct Lazy<'a> {
  pub(crate) pool: &'a StringPool,
  pub(crate) pool_keys: keys::PoolKeys,
  pub(crate) settings: Settings,
  pub(crate) std: Option<ModuleReference>,
  pub(crate) modules: Vec<Module>,
  pub(crate) functions: Vec<Function>,
  pub(crate) tokens: Vec<Vec<token::TokenSpan>>,
}

impl<'a> Lazy<'a> {
  pub fn new(pool: &'a StringPool, settings: Settings) -> Self {
    let lazy = Self {
      pool,
      settings,
      std: None,
      pool_keys: keys::PoolKeys::init(pool),
      modules: vec![],
      functions: vec![],
      tokens: vec![],
    };

    let argv = lazy.settings.argv.iter()
      .map(|arg| format_argument(arg))
      .collect::<Vec<_>>()
      .join(" ");

    print_message!(&lazy, {
      level: Debug,
      force: false,
      description: argv,
      contents: MessageContents::None,
    });

    lazy
  }

  /// Sounds like a rough time.
  ///
  /// Returns a [`ModuleReference`] to the standard library, tokenizing those
  /// structures if necessary
  pub(in crate) fn get_std(&mut self) -> Result<ModuleReference, LazyError> {
    if let Some(std) = self.std {
      return Ok(std);
    };

    print_message!(self, {
      level: Stub,
      force: false,
      description: line_dbg!("@std can only be imported from cwd").into(),
      contents: MessageContents::None,
    });

    let std_reference = self.add_file("@std", "std".into(), None)?;

    Ok(*self.std.insert(std_reference))
  }

  /// Creates a module with the provided values.  This module's
  /// [`ModuleParent`] will be [`ModuleParent::Path`] (from `path`) and this
  /// path will be resolved either using the provided path in `relative_to`, or
  /// the current working directory using [`std::env::current_dir`].
  ///
  /// The path will be validated and then the source code will be parsed for
  /// tokens and AST.  If successful, the corresponding [`ModuleReference`] will
  /// be returned.
  pub fn add_file(&mut self, name: &str, mut path: PathBuf, relative_to: Option<&Path>) -> Result<ModuleReference, LazyError> {
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
      contents: MessageContents::None,
    });

    // Check if our file is actually there
    if !path.is_file() {
      return Err(LazyError::NotExist(path));
    };

    let parent = |tokens, module| ModuleParent::Path(ModulePath { path, tokens, module });
    let module = self.create_module(name, parent);

    // Tokenize, asterize (parse AST)
    crate::aster::asterize(self, module)?;

    Ok(module)
  }

  pub fn create_module(&mut self, name: &str, parent: impl FnOnce(TokensId, ModuleReference) -> ModuleParent) -> ModuleReference {
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

  pub fn create_function(&mut self, module: ModuleReference, header: FunctionHeader) -> FunctionReference {
    let function_id = FunctionReference(self.functions.len());
    let function = Function::new(function_id, module, header);

    self.functions.push(function);
    self.rget_mut(module).functions.push(function_id);

    function_id
  }

  pub fn describe_module(&self, ModuleReference(id): ModuleReference) -> String {
    let module = self.modules.get(id).unwrap();
    let name = self.pool.get(module.name);

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
