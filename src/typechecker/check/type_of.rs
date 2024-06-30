
use crate::{tokenizer, typechecker::lang};

pub(super) trait TypeOf {
  fn type_of(&self) -> Option<lang::Type>;
  fn is_resolved(&self) -> bool;

  // fn type_of_expect(&self) -> Result<Type, TypeCheckerError> {
  //   let Some(ty) = self.type_of() else {
  //     todo!()
  //   };

  //   Ok(ty)
  // }
}

impl TypeOf for lang::TypeCell {
  fn is_resolved(&self) -> bool {
    match &*self.as_ref().borrow() {
      lang::Type::Intrinsic(_) => true,
      | lang::Type::UnsizedArrayOf(ty)
      | lang::Type::SizedArrayOf { ty, .. }
      | lang::Type::ReferenceTo { ty, .. }
      | lang::Type::Shared(ty) => ty.is_resolved(),
      lang::Type::Function { args, return_ty } => {
        if args.iter().any(|arg| !arg.is_resolved()) {
          return false;
        };

        return_ty.is_resolved()
      },
      lang::Type::Struct(tys) => tys.iter().all(|ty| ty.is_resolved()),
      | lang::Type::Unresolved { .. }
      // TODO: are these technically resolved?
      //       or should this be caught in a later stage
      | lang::Type::FuzzyInteger
      | lang::Type::Unknown => false,
    }
  }

  fn type_of(&self) -> Option<lang::Type> {
    match self.is_resolved() {
      true => Some(lang::Type::Shared(self.to_owned())),
      false => None,
    }
  }
}

impl TypeOf for lang::VariableReference {
  fn is_resolved(&self) -> bool {
    self.get().ty.is_resolved()
  }

  fn type_of(&self) -> Option<lang::Type> {
    self.get().ty.type_of()
  }
}

impl TypeOf for tokenizer::Literal {
  fn is_resolved(&self) -> bool {
    todo!()
  }

  fn type_of(&self) -> Option<lang::Type> {
    Some({
      match self {
        tokenizer::Literal::Integer(_) => lang::Type::FuzzyInteger,
        tokenizer::Literal::FloatingPoint(_) => todo!(),
        tokenizer::Literal::UnicodeString(_) => lang::Type::Struct(vec![
          // size:
          lang::Type::Intrinsic(lang::intrinsics::USIZE).into(),
          lang::Type::ReferenceTo {
            r#mut: false,
            ty: lang::Type::UnsizedArrayOf(
              lang::Type::Intrinsic(lang::intrinsics::UNICODE_CHAR).into()
            ).into(),
          }.into(),
        ]),
        tokenizer::Literal::CString(_) => lang::Type::ReferenceTo {
          r#mut: false,
          ty: lang::Type::UnsizedArrayOf(
            lang::Type::Intrinsic(lang::intrinsics::C_CHAR).into()
          ).into(),
        }.into(),
        tokenizer::Literal::ByteString(_) => todo!(),
        tokenizer::Literal::UnicodeChar(_) => todo!(),
        tokenizer::Literal::ByteChar(_) => todo!(),
      }
    })
  }
}

impl TypeOf for lang::Instruction {
  fn is_resolved(&self) -> bool {
    match self {
      lang::Instruction::Assign { dest, value } => {
        dest.is_resolved() && value.is_resolved()
      },
      lang::Instruction::Call { func, args } => {
        func.is_resolved() && args.iter().all(|arg| arg.is_resolved())
      },
      lang::Instruction::Literal(_) => true,
      lang::Instruction::Return(value) => value.is_resolved(),
    }
  }

  fn type_of(&self) -> Option<lang::Type> {
    match self {
      | lang::Instruction::Return(_)
      | lang::Instruction::Assign { .. } => Some(lang::Type::Intrinsic(lang::intrinsics::Intrinsic::Void)),
      lang::Instruction::Call { .. } => todo!(),
      lang::Instruction::Literal(literal) => literal.type_of(),
    }
  }
}

impl TypeOf for lang::Value {
  fn is_resolved(&self) -> bool {
    match self {
      lang::Value::Variable(var) => var.is_resolved(),
      lang::Value::Instruction(inst) => inst.is_resolved(),
    }
  }

  fn type_of(&self) -> Option<lang::Type> {
    match self {
      lang::Value::Variable(var) => var.get().ty.type_of(),
      lang::Value::Instruction(inst) => inst.type_of(),
    }
  }
}
