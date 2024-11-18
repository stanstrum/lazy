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

impl Resolve for FunctionBlock {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    for instruction in self.children.iter() {
      instruction.resolve(mods)?;

      if let Instruction::Return { value, .. } = &*instruction.borrow() {
        let x= self.parent().unwrap().upgrade().unwrap();
        let return_ty = &x.borrow().return_ty;
        if let Some(value) = value {
          value.coerce_with(return_ty, mods)?;
        } else {
          let parent = Rc::downgrade(self.parent().unwrap().upgrade().unwrap().scope_parent().unwrap().upgrade().as_ref().unwrap()).into();
          Type::Intrinsic { kind: Intrinsic::Void, parent }.coerce(return_ty, mods)?;
        }
      };
    };
    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    for instruction in self.children.iter() {
      instruction.ensure_resolved(compiler)?;
    }
    ok
  }
}

impl Resolve for Function {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    for argument in self.arguments.iter() {
      argument.resolve(mods)?;
    }

    self.return_ty.resolve(mods)?;
    self.body.resolve(mods)?;

    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    for argument in self.arguments.iter() {
      argument.ensure_resolved(compiler)?;
    }

    self.return_ty.ensure_resolved(compiler)?;
    self.body.ensure_resolved(compiler)?;

    ok
  }
}
