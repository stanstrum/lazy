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
    }
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

impl Resolve for LiteralInstruction {
  fn resolve(&self, _mods: &mut Modifications) -> Result {
    warn!("{}: LiteralInstruction::resolve", enchant!("stub"));
    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    self.ty.ensure_resolved(compiler)
  }
}

impl Resolve for BlockInstruction {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    for instruction in self.instructions.iter() {
      instruction.resolve(mods)?;
    }
    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    for instruction in self.instructions.iter() {
      instruction.ensure_resolved(compiler)?;
    }
    ok
  }
}

impl Resolve for Instruction {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    match self {
      Instruction::Literal(literal_instruction) => literal_instruction.resolve(mods),
      Instruction::Block(block_instruction) => block_instruction.resolve(mods),
      Instruction::Return { value, parent } => {
        let parent = parent.as_ref()
          .upgrade()
          .as_ref()
          .and_then(ScopeParent::scope_parent)
          .as_ref()
          .and_then(Weak::upgrade)
          .unwrap();
        let module = parent.scope_parent().unwrap();
        let return_type = parent.borrow().return_ty.clone();

        if let Some(value) = value {
          value.resolve(mods)?;
          return_type.coerce(value, mods)?;
        } else {
          return_type.coerce_with(
            &Type::Intrinsic {
              kind: Intrinsic::Void,
              parent: module.into(),
            },
            mods,
          )?;
        };
        ok
      },
      Instruction::ImplicitReturnLast { value, .. } => {
        value.resolve(mods)?;
        ok
      },
    }
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    match self {
      Instruction::Literal(literal_instruction) => literal_instruction.ensure_resolved(compiler),
      Instruction::Block(block_instruction) => block_instruction.ensure_resolved(compiler),
      | Instruction::Return { value: Some(value), .. }
      | Instruction::ImplicitReturnLast { value, .. } => value.ensure_resolved(compiler),
      Instruction::Return { value: None, .. } => ok,
    }
  }
}
