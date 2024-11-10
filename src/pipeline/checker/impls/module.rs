use super::*;

impl Resolve for ModuleChild {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    match self {
      ModuleChild::Function(rc) => rc.borrow().resolve(mods),
      ModuleChild::Module(rc) => rc.borrow().resolve(mods),
    }
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    match self {
      ModuleChild::Function(rc) => rc.borrow().ensure_resolved(compiler),
      ModuleChild::Module(rc) => rc.borrow().ensure_resolved(compiler),
    }
  }
}

impl Resolve for Module {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    for child in self.children.iter() {
      child.borrow().resolve(mods)?;
    };

    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    for child in self.children.iter() {
      child.borrow().ensure_resolved(compiler)?;
    };

    ok
  }
}

