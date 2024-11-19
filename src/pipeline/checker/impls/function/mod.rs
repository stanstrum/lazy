mod coerce;
mod resolve;

use super::*;

impl LiteralInstructionKind {
  pub(super) fn type_of<T: Into<OpaqueParent<WeakCell<Module>>>>(&self, parent: T) -> Type<Module> {
    let parent = parent.into();

    match self {
      LiteralInstructionKind::Integer(_) => Type::new_union([
        Type::new_intrinsic(Intrinsic::I8, parent.clone()),
        Type::new_intrinsic(Intrinsic::I16, parent.clone()),
        Type::new_intrinsic(Intrinsic::I32, parent.clone()),
        Type::new_intrinsic(Intrinsic::I64, parent.clone()),
        Type::new_intrinsic(Intrinsic::U8, parent.clone()),
        Type::new_intrinsic(Intrinsic::U16, parent.clone()),
        Type::new_intrinsic(Intrinsic::U32, parent.clone()),
        Type::new_intrinsic(Intrinsic::U64, parent.clone()),
        Type::new_intrinsic(Intrinsic::F32, parent.clone()),
        Type::new_intrinsic(Intrinsic::F64, parent),
      ]),
      LiteralInstructionKind::Float(_) => todo!(),
      LiteralInstructionKind::String(_) => todo!(),
    }
  }
}

impl TypeOf for LiteralInstruction {
  fn type_of(&self) -> Type<Module> {
    Type::Reference(new_rc_cell(Reference::Resolved(self.ty.clone())))
  }
}

impl TypeOf for Instruction {
  fn type_of(&self) -> Type<Module> {
    match self {
      Instruction::Literal(literal_instruction) => literal_instruction.type_of(),
      Instruction::Block(rc) => rc.borrow().type_of(),
      Instruction::Return { .. } => todo!(),
      Instruction::ImplicitReturnLast { .. } => todo!(),
    }
  }
}

impl TypeOf for BlockInstruction {
  fn type_of(&self) -> Type<Module> {
    Type::Reference(new_rc_cell(Reference::Resolved(self.out.clone())))
  }
}
