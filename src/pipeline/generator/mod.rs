use inkwell::context::Context;

use crate::compiler::workflow::DefaultWorkflow;
use crate::Result;

use crate::compiler::{
  Compiler,
  CompilerStoreHandle,
  CompilerWorkflow,
  Generate,
};

use crate::translator::lang::{RcCell, Module};

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Generator<W: CompilerWorkflow> {
  handle: CompilerStoreHandle<W>,
  input: RcCell<Module>,
  context: Context,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct GeneratorModule {
  context: Context,
  module: inkwell::module::Module<'static>,
}

trait LlvmGenerate<W: CompilerWorkflow> {
  type Out;

  fn generate_in_context(&self, context: &Context) -> Result<Self::Out>;
}

impl LlvmGenerate<DefaultWorkflow> for Module {
  type Out = inkwell::module::Module<'static>;

  fn generate_in_context(&self, context: &Context) -> Result<Self::Out> {
    let module = context.create_module(format!("{:?}", &self.name).as_str());

    // TODO: ???
    Ok(unsafe { std::mem::transmute(module) })
  }
}

impl<L: LlvmGenerate<W>, W: CompilerWorkflow> LlvmGenerate<W> for RcCell<L> {
  type Out = L::Out;

  fn generate_in_context(&self, context: &Context) -> Result<Self::Out> {
    self.borrow().generate_in_context(context)
  }
}

impl Generate<DefaultWorkflow> for Generator<DefaultWorkflow> {
  type In = RcCell<Module>;
  type Out = inkwell::module::Module<'static>;

  fn new(input: Self::In, handle: CompilerStoreHandle<DefaultWorkflow>) -> Self {
    Self {
      input,
      handle,
      context: Context::create(),
    }
  }

  fn generate(self, compiler: &mut Compiler<DefaultWorkflow>) -> Result<Self::Out> {
    let module = compiler.context.create_module(format!("{:?}", &self.input.borrow().name).as_str());

    Ok(unsafe { std::mem::transmute(module) })
  }
}
