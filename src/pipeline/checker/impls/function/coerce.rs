use super::*;

impl CoerceWith<Type<Module>> for LiteralInstruction {
  fn coerce_with(&self, with: &Type<Module>, mods: &mut Modifications) -> Result {
    self.ty.coerce_with(with, mods)
  }
}

impl CoerceWith<RcCell<Type<Module>>> for RcCell<BlockInstruction> {
  fn coerce_with(&self, with: &RcCell<Type<Module>>, mods: &mut Modifications) -> Result {
    let this = self.borrow();

    for instruction in this.instructions.iter() {
      if let Instruction::Return { value, .. } = &*instruction.borrow() {
        if let Some(value) = value {
          value.coerce_with(with, mods)?;
        } else {
          let parent = this
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

impl CoerceWith<RcCell<Type<Module>>> for RcCell<Instruction> {
  fn coerce_with(&self, with: &RcCell<Type<Module>>, mods: &mut Modifications) -> Result {
    match &*self.borrow() {
      Instruction::Literal(literal_instruction) => literal_instruction.coerce_with(with, mods),
      Instruction::Block(block_instruction) => block_instruction.coerce_with(with, mods),
      Instruction::Return { .. } => todo!(),
      Instruction::ImplicitReturnLast { .. } => todo!(),
    }
  }
}
