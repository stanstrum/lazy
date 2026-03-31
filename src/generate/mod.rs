mod context;
pub mod args;
mod compile;

mod types;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::lang;
use crate::lang::span::GetSpan;
use crate::lang::reference::{Reference, Store};
use crate::resolve::TypeOf;
use crate::tokenize::token;

use {args::*, context::*};

#[derive(Debug)]
pub enum Error {
  StillUnresolved {
    what: String,
    note: String,
    span: token::Span,
  },
}

type Result<T = ()> = std::result::Result<T, Error>;

struct Compilation<'lazy, 'pool, 'llvm> {
  lazy: &'lazy lang::Lazy<'pool>,
  llvm: LLVMContext<'llvm>,
  functions: HashMap<
    lang::reference::FunctionReference,
    inkwell::values::FunctionValue<'llvm>,
  >,
}

impl<'lazy, 'pool, 'llvm> Compilation<'lazy, 'pool, 'llvm> {
  fn new(lazy: &'lazy lang::Lazy<'pool>, context: LLVMContext<'llvm>) -> Self {
    Self {
      lazy,
      llvm: context,
      functions: HashMap::new(),
    }
  }

  fn get_or_declare_function(&mut self,
    function: lang::reference::FunctionReference,
  ) -> Result<inkwell::values::FunctionValue<'llvm>> {
    // return it if we have it already
    if let Some(value) = self.functions.get(&function) {
      return Ok(*value);
    };

    // if we're still here, we need to make and store the function
    let name = {
      let borrow = function.rget_from(self.lazy);
      let name_id = borrow.header.name.id;
      self.lazy.pool.get(name_id).collect::<String>()
    };

    let function_ty = types::make_function_type(self, function)?;

    // SPONGE: we need to determine this from the AST
    let linkage = Some(inkwell::module::Linkage::External);

    // add the function
    let function_value = self.llvm.module.add_function(&name, function_ty, linkage);

    // store to our cache
    self.functions.insert(function, function_value);

    Ok(function_value)
  }
}

pub(super) struct Program {
  context: inkwell::context::Context,
  global: lang::reference::ModuleReference,
  cli_args: CliArgs,
}

pub(super) struct ProgramCompilation<'ctx> {
  program: &'ctx Program,
  llvm: LLVMContext<'ctx>,
}

pub(super) struct ProgramObjectFile {
  target: String,
  path: tempfile::TempPath,
}

pub(super) struct ProgramExecutable {
  path: PathBuf,
}

impl Program {
  pub(super) fn new(global: lang::reference::ModuleReference, cli_args: CliArgs) -> Self {
    Self {
      context: inkwell::context::Context::create(),
      global,
      // SPONGE: need to parse this struct from settings
      cli_args,
    }
  }

  pub(super) fn compile<'ctx>(&'ctx self, lazy: &lang::Lazy) -> Result<ProgramCompilation<'ctx>> {
    let llvm_ctx = LLVMContext::new(&self.context, &self.cli_args);
    let mut comp = Compilation::new(lazy, llvm_ctx);

    compile::compile_module(&mut comp, self.global)?;

    Ok(ProgramCompilation {
      program: self,
      llvm: comp.llvm,
    })
  }
}

impl<'ctx> ProgramCompilation<'ctx> {
  pub(super) fn save_to_file(self, file_type: inkwell::targets::FileType) -> Result<ProgramObjectFile> {
    let file = {
      let temp_file_result = tempfile::Builder::new()
      // .prefix("lazy-object-")
      .suffix(".o")
      .append(false)
      .tempfile();

      match temp_file_result {
        Ok(file) => file,
        Err(err) => todo!("tempfile err {err:?}"),
      }
    };

    let path = file.into_temp_path();

    if let Err(err) = self.llvm.machine.write_to_file(
      &self.llvm.module,
      file_type,
      &path,
    ) {
      panic!("LLVM error: {err}");
    };

    let target = self.llvm.machine.get_triple().as_str().to_string_lossy().to_string();

    Ok(ProgramObjectFile {
      path,
      target,
    })
  }

  pub(super) fn debug(&self) {
    self.llvm.dump_module();
  }

  pub(super) fn optimize(&mut self) {
    self.llvm.run_passes(&self.program.cli_args.passes);
  }
}

impl ProgramObjectFile {
  pub(super) fn link_with<'a>(self, out_path: &'a Path, linked: &[&str]) -> Result<&'a Path> {
    let out_dir = if out_path.is_absolute() {
      out_path.parent()
        .expect("a parent directory in output")
        .to_owned()
    } else {
      std::env::current_dir().expect("cwd")
    };

    let path = self.path;

    // let output = out_path.file_name()
    //   .map(|path| path.to_str().expect("out_path to have a file name"))
    //   .unwrap_or("a.out");

    // SPONGE: may be possible to inject arguments this way?
    let links = linked.iter()
      .map(|name| format!("-l{name}"));

    let mut builder = cc::Build::new();

    let compiler = builder
      .object(&path)
      .out_dir(out_dir)
      .flags(links)
      .target(&self.target)
      .host(&self.target)
      .opt_level_str("2")
      .env("LC_ALL", "C")
      .cargo_debug(false)
      .cargo_metadata(false)
      .cargo_output(false)
      .cargo_warnings(false)
      .get_compiler()
    ;

    let temp_path_str = path.to_str().expect("to convert path to str");
    let out_path_str = out_path.to_str().expect("to convert path to str");

    let compiler_result = compiler.to_command()
      .args(compiler.args())
      .args(["-o", out_path_str])
      .arg(temp_path_str)
      .spawn()
      .expect("to spawn compiler subprocess")
      .wait()
      .expect("to wait for compiler subproccess")
    ;

    assert!(compiler_result.success(), "compiler subprocess errored");
    assert!(out_path.exists(), "didn't create output file");

    path.close().expect("to close temp file");

    Ok(out_path)
  }
}
