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

impl Program {
  pub(super) fn new(global: lang::reference::ModuleReference, cli_args: CliArgs) -> Self {
    Self {
      context: inkwell::context::Context::create(),
      global,
      // SPONGE: need to parse this struct from settings
      cli_args,
    }
  }

  pub(super) fn compile<'lazy, 'ctx>(&'ctx self, lazy: &'lazy lang::Lazy) -> Result<ProgramCompilation<'ctx>> {
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
  pub(super) fn save_to_file(self, file_type: inkwell::targets::FileType, out_path: &Path) -> Result {
    if let Err(err) = self.llvm.machine.write_to_file(
      &self.llvm.module,
      file_type,
      out_path,
    ) {
      panic!("LLVM error: {err}");
    };

    Ok(())
  }

  pub(super) fn debug(&self) {
    self.llvm.dump_module();
  }

  pub(super) fn optimize(&mut self) {
    self.llvm.run_passes(&self.program.cli_args.passes);
  }

  pub(super) fn run(&self) -> Result {
    todo!()
  }
}
