use super::*;
use crate::translator::ScopeParent;

impl Resolve for FunctionArgument {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    self.ty.resolve(mods)?;

    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    self.ty.ensure_resolved(compiler)
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

impl<C: CoerceWith<T>, T> CoerceWith<RcCell<T>> for C {
  fn coerce_with(&self, with: &RcCell<T>, mods: &mut Modifications) -> Result {
    self.coerce_with(&*with.borrow(), mods)
  }
}

impl CoerceWith<Type<Module>> for LiteralInstruction {
  fn coerce_with(&self, with: &Type<Module>, mods: &mut Modifications) -> Result {
    self.ty.coerce_with(with, mods)
  }
}

impl CoerceWith<Type<Module>> for RcCell<Type<Module>> {
  fn coerce_with(&self, with: &Type<Module>, mods: &mut Modifications) -> Result {
    match (&*self.borrow(), with) {
      (Type::Reference(reference), _) => {
        if let Some(reference) = reference.get_and_maybe_modify(mods)?.as_ref().and_then(Weak::upgrade) {
          reference.coerce_with(with, mods)?;
        } else {
          warn!("{}: couldn't coerce type because reference was unresolved", enchant!("coerce_with"));
        };
      },
      (_, Type::Reference(reference)) => {
        if let Some(reference) = reference.get_and_maybe_modify(mods)?.as_ref().and_then(Weak::upgrade) {
          reference.coerce(self, mods)?;
        } else {
          warn!("{}: couldn't coerce type because reference was unresolved", enchant!("coerce_with"));
        };
      },
      (Type::Intrinsic { kind: a, .. }, Type::Intrinsic { kind: b, .. }) => {
        if a != b {
          panic!("coerce_with failed");
        };
      },
      (Type::UnresolvedInstrinsic(weak), other) => {
        match weak.upgrade().unwrap().as_ref() {
          LiteralInstructionKind::Integer(value) => todo!(),
          LiteralInstructionKind::Float(value) => todo!(),
          LiteralInstructionKind::String(value) => todo!(),
        };
      },
      other => todo!("{other:#?}"),
    };
    ok
  }
}

impl CoerceWith<Type<Module>> for Type<Module> {
  fn coerce_with(&self, with: &Type<Module>, mods: &mut Modifications) -> Result {
    match (self, with) {
      (Type::Intrinsic { kind: a, .. }, Type::Intrinsic { kind: b, .. }) => {
        if a != b {
          panic!("coerce fail");
        };

        ok
      },
      other => todo!("{other:#?}"),
    }
  }
}

impl CoerceWith<Type<Module>> for BlockInstruction {
  fn coerce_with(&self, with: &Type<Module>, mods: &mut Modifications) -> Result {
    for instruction in self.instructions.iter() {
      if let Instruction::Return { value, .. } = &*instruction.borrow() {
        if let Some(value) = value {
          value.coerce_with(with, mods)?;
        } else {
          let parent = self
            .parent().unwrap().upgrade().unwrap()
            .scope_parent().unwrap().upgrade().unwrap()
            .scope_parent().unwrap();

          Type::Intrinsic {
            kind: Intrinsic::Void,
            parent: parent.into(),
          }.coerce(with, mods)?;
        };
      };
    };
    ok
  }
}

impl CoerceWith<Type<Module>> for RcCell<Instruction> {
  fn coerce_with(&self, with: &Type<Module>, mods: &mut Modifications) -> Result {
    match &*self.borrow() {
      Instruction::Literal(literal_instruction) => literal_instruction.coerce_with(with, mods),
      Instruction::Block(block_instruction) => block_instruction.coerce_with(with, mods),
      Instruction::Return { parent, value } => todo!(),
    }
  }
}

impl TypeOf for Instruction {
  fn type_of(&self) -> Type<Module> {
    todo!()
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
    }
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    todo!()
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
