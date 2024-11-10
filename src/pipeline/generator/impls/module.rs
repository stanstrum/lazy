use super::*;

impl lang::ModuleChild {
  fn generate_in_module<W: CompilerWorkflow>(&self, generator: &mut Generator<W>, module: &inkwell::module::Module) -> Result {
    match self {
      lang::ModuleChild::Function(rc) => lang::Function::generate_in_module(rc, generator, module),
      lang::ModuleChild::Module(rc) => rc.borrow().generate_in_module(generator, module),
    }
  }
}

impl lang::Module {
  pub(super) fn generate_in_module<W: CompilerWorkflow>(&self, generator: &mut Generator<W>, module: &inkwell::module::Module) -> Result {
    for child in self.children.iter() {
      child.borrow().generate_in_module(generator, module)?;
    };

    ok
  }
}
