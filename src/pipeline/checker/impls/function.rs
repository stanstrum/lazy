use super::*;

impl Resolve for FunctionArgument {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    self.ty.resolve(mods)?;

    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    self.ty.ensure_resolved(compiler)
  }
}

impl Resolve for Function {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    for argument in self.arguments.iter() {
      argument.resolve(mods)?;
    };

    self.return_ty.resolve(mods)?;

    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    for argument in self.arguments.iter() {
      argument.ensure_resolved(compiler)?;
    };

    self.return_ty.ensure_resolved(compiler)?;

    ok
  }
}
