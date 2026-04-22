pub mod keys;

use string_pool::StringPool;

use crate::lang::LazyError;
use crate::lang::function::{Function, FunctionHeader};
use crate::lang::module::{Module, ModuleParent, ModulePath, TokensId};
use crate::lang::reference::{FunctionReference, ModuleReference, Store};
use crate::settings::Settings;
use crate::settings::format::format_argument;
use crate::tokenize::token;
use crate::{line_dbg, print_message};

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::os::unix::fs::PermissionsExt;

use crate::error;

#[derive(Debug)]
pub struct Lazy<'pool> {
  pub(crate) pool: &'pool StringPool,
  pub(crate) pool_keys: keys::PoolKeys,
  pub(crate) settings: Settings,
  pub(crate) std: Option<ModuleReference>,
  pub(crate) modules: Vec<Module>,
  pub(crate) functions: Vec<Function>,
  pub(crate) tokens: Vec<Vec<token::TokenSpan>>,
}

impl<'pool> Lazy<'pool> {
  pub fn check(&mut self) -> Result<ModuleReference, error::PrintableMessage> {
    // Instantiate the global scope
    let path = self.settings.input_path.to_owned();
    let global = self.add_file("@global", path, None)?;

    // Resolve, verify
    crate::resolve::resolve_and_verify(self, global)?;

    // Debugs
    crate::debug::source(self, &global);
    crate::debug::string_pool(self);

    Ok(global)
  }

  pub fn build<'a>(&'a mut self) -> Result<&'a Path, error::PrintableMessage> {
    let global = self.check()?;

    // Otherwise, let's go build the module
    let args = crate::generate::args::CliArgs {
      target: None,
      opt_level: crate::generate::args::OptimizationLevel::O0,
      passes: "instcombine,reassociate,gvn,simplifycfg,mem2reg".into(),
    };

    let program = crate::generate::Program::new(global, args);
    let compilation = program.compile(self)?;

    // Debug the LLVM source
    crate::debug::llvm_source(self, &compilation);

    // Optimize the IR
    compilation.optimize(self)?;
    crate::debug::llvm_source(self, &compilation);

    // Write out the object file for the global module
    // TODO: get this from settings
    let file_type = inkwell::targets::FileType::Object;
    let object_file = compilation.save_to_file(file_type)?;

    crate::debug::object_file(self, &object_file);

    // TODO: find out what needs to be linked
    let linked = [
      "c", // libc
    ];

    // Link the module and write the program to the output file
    let executable = object_file.link_with(&self.settings.output_path, &linked)?;

    // Set the output file's permissions to be executable
    let perms = std::fs::Permissions::from_mode(0o755);
    std::fs::set_permissions(executable, perms)
      .expect("failed to chmod 755 {executable:?}");

    Ok(executable)
  }

  pub fn run(&mut self) -> Result<ExitCode, error::PrintableMessage> {
    let executable = self.build()?;

    // Otherwise, go run the child program
    let mut command = std::process::Command::new(executable);
    crate::debug::subprocess_command(self, &command);

    let mut child = command
      // .args(args);
      .spawn()
      .expect("to launch {executable:?}");

    // Wait on the child and get an exit status
    let exit_status = child.wait()
      .expect("to wait on child process");

    // Print that info and set our own exit code accordingly
    let (exit_code, level, message) = if exit_status.success() {
      (
        ExitCode::SUCCESS, error::Level::Info,
        "Program exited successfully.".into(),
      )
    } else if let Some(code) = exit_status.code() {
      (
        ExitCode::FAILURE, error::Level::Error,
        format!("Program exited with status code {code}."),
      )
    } else {
      (
        ExitCode::FAILURE, error::Level::Error,
        "Program exited unsuccessfully.".into(),
      )
    };

    print_message!(self, {
      level,
      force: false,
      description: message,
      contents: MessageContents::None,
    });

    Ok(exit_code)
  }

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
