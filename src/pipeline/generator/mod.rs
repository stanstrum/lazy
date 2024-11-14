mod impls;
mod shell;

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use inkwell::basic_block::BasicBlock;
use inkwell::context::Context;
use inkwell::values::{FunctionValue, PointerValue};
use tempfile::NamedTempFile;

use crate::compiler::{
  error::*, workflow::DefaultWorkflow, Compiler, CompilerStoreHandle, CompilerWorkflow, Generate,
};
use crate::translator::lang::{Module, RcCell};
use crate::{enchant, ok, Result};

#[derive(Debug)]
pub(crate) struct BlockData {
  block: BasicBlock<'static>,
  result: Option<PointerValue<'static>>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Generator<W: CompilerWorkflow> {
  handle: CompilerStoreHandle<W>,
  input: Option<RcCell<Module>>,
  context: Context,
  functions: Vec<FunctionValue<'static>>,
  blocks: Vec<BlockData>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct GeneratorModule {
  context: Context,
  module: inkwell::module::Module<'static>,
}

impl Generate<DefaultWorkflow> for Generator<DefaultWorkflow> {
  type In = RcCell<Module>;
  type Out = PathBuf;

  fn new(input: Self::In, handle: CompilerStoreHandle<DefaultWorkflow>) -> Self {
    Self {
      input: Some(input),
      handle,
      context: Context::create(),
      functions: vec![],
      blocks: vec![],
    }
  }

  fn generate(mut self, compiler: &mut Compiler<DefaultWorkflow>) -> Result<Self::Out> {
    let input = self.input.take().unwrap();
    let module = input.borrow().generate(&mut self, &compiler.context)?;

    if compiler.settings.print_llvm {
      info!(
        "{}: {}:\n{}",
        enchant!("--print-llvm"),
        self.handle.proper_name(compiler),
        module.print_to_string().to_string_lossy().trim()
      );
    };

    // verify the module using the inkwell api -- this error is easier to handle
    // here rather than an error that comes from llc's stderr
    if let Err(err) = module.verify() {
      return IOSnafu { err: err.to_string() }.fail()?;
    };

    self.generate_object_file(compiler, module)
  }
}
