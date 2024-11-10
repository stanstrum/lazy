mod module;
mod type_methods;
mod r#type;
mod function;

use inkwell::context::ContextRef;
use inkwell::types::{
  BasicMetadataTypeEnum,
  BasicTypeEnum,
  FunctionType,
  MetadataType,
  VoidType,
};

use super::*;
use type_methods::*;

use crate::ok;
use crate::translator::lang;

trait TypeOf {
  type Out<'ctx>;

  fn g_type_of<'a, 'ctx: 'a>(&self, context: &'a ContextOrRef<'a, 'ctx>) -> Result<Self::Out<'ctx>>;
}

impl<T: TypeOf> TypeOf for RcCell<T> {
  type Out<'ctx> = T::Out<'ctx>;

  fn g_type_of<'a, 'ctx: 'a>(&self, context: &'a ContextOrRef<'a, 'ctx>) -> Result<Self::Out<'ctx>> {
    self.borrow().g_type_of(context)
  }
}

impl lang::Module {
    pub(in crate::pipeline::generator) fn generate<
    'ctx,
    W: CompilerWorkflow
  >(&self, generator: &mut Generator<W>, context: &'ctx Context) -> Result<inkwell::module::Module<'ctx>> {
    let module = context.create_module(format!("{:?}", &self.name).as_str());

    self.generate_in_module(generator, &module)?;

    Ok(module)
  }
}
