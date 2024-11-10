use super::*;

impl lang::ModuleChild {
  fn generate_in_module<'a, 'ctx: 'a, W: CompilerWorkflow>(&self, generator: &'a mut Generator<W>, module: &'a inkwell::module::Module<'ctx>) -> Result {
    match self {
      lang::ModuleChild::Function(rc) => lang::Function::generate_in_module(rc, generator, module),
      lang::ModuleChild::Module(rc) => rc.borrow().generate_in_module(generator, module),
    }
  }
}

impl lang::Module {
  pub(super) fn generate_in_module<'a, 'ctx: 'a, W: CompilerWorkflow>(&'a self, generator: &'a mut Generator<W>, module: &'a inkwell::module::Module<'ctx>) -> Result {
    for child in self.children.iter() {
      child.borrow().generate_in_module(generator, module)?;
    };

    ok
  }
}
