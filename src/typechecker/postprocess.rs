use crate::tokenizer;
use crate::typechecker::{
  lang,
  lang::intrinsics::Intrinsic,
  Domain,
  DomainMember,
  Program,
  TypeChecker,
  TypeCheckerError,
};

pub(super) trait Postprocess {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError>;
}

impl Postprocess for lang::Variable {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    self.ty.postprocess(checker)
  }
}

impl Postprocess for lang::Value {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    match self {
      lang::Value::Variable(variable) => variable.get().postprocess(checker),
      lang::Value::Instruction(instruction) => instruction.postprocess(checker),
      lang::Value::Literal { ty, .. } => ty.postprocess(checker),
    }
  }
}

impl Postprocess for lang::Instruction {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    match self {
      lang::Instruction::Assign { dest, value, .. } => {
        dest.postprocess(checker)?;
        value.postprocess(checker)
      },
      lang::Instruction::Call { func, args, .. } => {
        func.postprocess(checker)?;

        for arg in args.iter_mut() {
          arg.postprocess(checker)?;
        };

        Ok(())
      },
      lang::Instruction::Return { value, to, .. } => {
        if let Some(value) = value {
          value.postprocess(checker)?;
        };

        to.postprocess(checker)?;

        Ok(())
      },
      lang::Instruction::Value(value) => value.postprocess(checker),
    }
  }
}

impl Postprocess for lang::Block {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    for variable in self.variables.borrow_mut().inner.iter_mut() {
      variable.postprocess(checker)?;
    };

    for instruction in self.body.iter_mut() {
      instruction.postprocess(checker)?;
    };

    Ok(())
  }
}

impl Postprocess for lang::Function {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    self.return_ty.postprocess(checker)?;

    for argument in self.arguments.inner.iter_mut() {
      argument.postprocess(checker)?;
    };

    self.body.postprocess(checker)?;

    Ok(())
  }
}

impl Postprocess for lang::Type {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    match self {
      | lang::Type::UnsizedArrayOf { ty, .. }
      | lang::Type::SizedArrayOf { ty, .. }
      | lang::Type::ReferenceTo { ty, .. }
      | lang::Type::Shared(ty) => ty.postprocess(checker),

      | lang::Type::Intrinsic { .. }
      | lang::Type::Unresolved { .. }
      | lang::Type::Unknown { .. } => Ok(()),

      lang::Type::Function { args, return_ty, .. } => {
        for arg in args.iter_mut() {
          arg.postprocess(checker)?;
        };

        return_ty.postprocess(checker)
      },
      lang::Type::Struct { members, .. } => {
        for member in members.iter_mut() {
          member.postprocess(checker)?;
        };

        Ok(())
      },

      lang::Type::FuzzyInteger { span } => {
        *self = lang::Type::Intrinsic {
          kind: Intrinsic::U64,
          span: *span,
        };

        Ok(())
      },
      lang::Type::FuzzyString { size, element_ty, span } => {
        let span = *span;

        *self = lang::Type::ReferenceTo {
          r#mut: false,
          ty: lang::Type::SizedArrayOf {
            count: lang::Value::Literal {
              literal: tokenizer::Literal {
                kind: tokenizer::LiteralKind::Integer(*size as u64),
                span,
              },
              ty: lang::Type::Intrinsic {
                kind: Intrinsic::U64,
                span,
              }.into(),
            },
            ty: lang::Type::Intrinsic {
              kind: element_ty.to_owned(),
              span,
            }.into(),
            span,
          }.into(),
          span,
        };

        Ok(())
      },
    }
  }
}

impl Postprocess for lang::TypeCell {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    self.borrow_mut().postprocess(checker)
  }
}

impl Postprocess for DomainMember {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    match self {
      DomainMember::Domain(domain) => domain.postprocess(checker),
      DomainMember::Function(function) => function.postprocess(checker),
      DomainMember::Type(r#type) => r#type.postprocess(checker),
    }
  }
}

impl Postprocess for Domain {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    for member in self.inner.values_mut() {
      member.postprocess(checker)?;
    };

    Ok(())
  }
}

impl Postprocess for Program {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    for data in self.inner.values_mut() {
      data.domain.postprocess(checker)?;
    };

    Ok(())
  }
}
