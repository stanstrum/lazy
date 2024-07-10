use crate::typechecker::lang;

use crate::tokenizer;
use crate::tokenizer::GetSpan;

pub(crate) trait TypeOf {
  fn type_of(&self) -> Option<lang::Type>;
  fn is_resolved(&self) -> bool;
}

impl TypeOf for lang::TypeCell {
  fn is_resolved(&self) -> bool {
    match &*self.as_ref().borrow() {
      lang::Type::Intrinsic { .. } => true,
      | lang::Type::UnsizedArrayOf { ty, .. }
      | lang::Type::SizedArrayOf { ty, .. }
      | lang::Type::ReferenceTo { ty, .. }
      | lang::Type::Shared(ty) => ty.is_resolved(),
      lang::Type::Function { args, return_ty, .. } => {
        if args.iter().any(|arg| !arg.is_resolved()) {
          return false;
        };

        return_ty.is_resolved()
      },
      lang::Type::Struct { members: tys, .. } => tys.iter().all(|ty| ty.is_resolved()),
      | lang::Type::Unresolved { .. }
      // TODO: are these technically resolved?
      //       or should this be caught in a later stage
      | lang::Type::FuzzyInteger { .. }
      | lang::Type::FuzzyString { .. }
      | lang::Type::Unknown { .. } => false,
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
        tokenizer::Literal { kind: tokenizer::LiteralKind::Integer(_), span } => lang::Type::FuzzyInteger { span: span.to_owned() },
        tokenizer::Literal { kind: tokenizer::LiteralKind::FloatingPoint(_), .. } => todo!(),
        tokenizer::Literal { kind: tokenizer::LiteralKind::UnicodeString(content), span } => lang::Type::FuzzyString {
          size: content.len(),
          element_ty: lang::intrinsics::UNICODE_CHAR,
          span: span.to_owned(),
        },
        tokenizer::Literal { kind: tokenizer::LiteralKind::ByteString(content), span } => lang::Type::FuzzyString {
          size: content.len(),
          element_ty: lang::intrinsics::Intrinsic::U8,
          span: span.to_owned(),
        },
        tokenizer::Literal { kind: tokenizer::LiteralKind::CString(content), span } => lang::Type::FuzzyString {
          // the `+ 1` is for the null-delimiter
          size: content.len() + 1,
          element_ty: lang::intrinsics::C_CHAR,
          span: span.to_owned(),
        },
        tokenizer::Literal { kind: tokenizer::LiteralKind::UnicodeChar(_), span } => lang::Type::Intrinsic {
          kind: lang::intrinsics::UNICODE_CHAR,
          span: span.to_owned()
        },
        // TODO: not sure if this should be signed or not
        tokenizer::Literal { kind: tokenizer::LiteralKind::ByteChar(_), span } => lang::Type::Intrinsic {
          kind: lang::intrinsics::Intrinsic::U8,
          span: span.to_owned(),
        },
      }
    })
  }
}

impl TypeOf for lang::Instruction {
  fn is_resolved(&self) -> bool {
    match self {
      lang::Instruction::Assign { dest, value, .. } => {
        dest.is_resolved() && value.is_resolved()
      },
      lang::Instruction::Call { func, args, .. } => {
        func.is_resolved() && args.iter().all(|arg| arg.is_resolved())
      },
      | lang::Instruction::Return { value, .. }
      | lang::Instruction::Value(value) => value.is_resolved(),
    }
  }

  fn type_of(&self) -> Option<lang::Type> {
    match self {
      | lang::Instruction::Return { .. }
      | lang::Instruction::Assign { .. } => Some(lang::Type::Intrinsic {
        kind: lang::intrinsics::Intrinsic::Void,
        span: self.get_span().to_owned(),
      }),
      lang::Instruction::Call { .. } => todo!(),
      lang::Instruction::Value(value) => value.type_of(),
    }
  }
}

impl TypeOf for lang::Value {
  fn is_resolved(&self) -> bool {
    match self {
      lang::Value::Variable(var) => var.is_resolved(),
      lang::Value::Instruction(inst) => inst.is_resolved(),
      lang::Value::Literal(lit) => lit.is_resolved(),
    }
  }

  fn type_of(&self) -> Option<lang::Type> {
    match self {
      lang::Value::Variable(var) => var.get().ty.type_of(),
      lang::Value::Instruction(inst) => inst.type_of(),
      lang::Value::Literal(lit) => lit.type_of(),
    }
  }
}

impl TypeOf for lang::Function {
  fn type_of(&self) -> Option<lang::Type> {
    if !self.return_ty.is_resolved() {
      return None;
    };

    let arguments = &self.arguments.inner;
    if arguments.iter().any(|var| !var.ty.is_resolved()) {
      return None;
    };

    let args = arguments.iter()
      .map(|var| var.ty.to_owned())
      .collect();

    let return_ty = self.return_ty.to_owned();
    let span = self.get_span();

    Some(lang::Type::Function { args, return_ty, span })
  }

  fn is_resolved(&self) -> bool {
    todo!()
  }
}
