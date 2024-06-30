use crate::typechecker::lang;

pub(super) trait TypeOf {
  fn type_of(&self) -> Option<lang::Type>;

  // fn type_of_expect(&self) -> Result<Type, TypeCheckerError> {
  //   let Some(ty) = self.type_of() else {
  //     todo!()
  //   };

  //   Ok(ty)
  // }
}

impl TypeOf for lang::Type {
  fn type_of(&self) -> Option<lang::Type> {
    match self {
      lang::Type::Intrinsic(_) => todo!(),
      lang::Type::Unresolved { .. } => todo!(),
      lang::Type::UnsizedArrayOf(_) => todo!(),
      lang::Type::SizedArrayOf { .. } => todo!(),
      lang::Type::ReferenceTo { .. } => todo!(),
      lang::Type::Shared(_) => todo!(),
      lang::Type::Function { .. } => todo!(),
      lang::Type::Unknown => todo!(),
    }
  }
}

impl TypeOf for lang::TypeCell {
  fn type_of(&self) -> Option<lang::Type> {
    self.borrow().type_of()
  }
}

impl TypeOf for lang::Value {
  fn type_of(&self) -> Option<lang::Type> {
    match self {
      lang::Value::Variable(var) => var.get().ty.type_of(),
      lang::Value::Instruction(_) => todo!(),
    }
  }
}
