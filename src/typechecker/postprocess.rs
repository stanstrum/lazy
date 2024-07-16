use crate::tokenizer;
use crate::typechecker::{
  lang,
  lang::intrinsics::Intrinsic,
  Domain,
  DomainMember,
  DomainMemberKind,
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

impl Postprocess for lang::ControlFlow {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    match self {
      | lang::ControlFlow::While { condition, body, .. }
      | lang::ControlFlow::DoWhile { condition, body, .. }
      | lang::ControlFlow::Until { condition, body, .. }
      | lang::ControlFlow::DoUntil { condition, body, .. } => {
        condition.postprocess(checker)?;
        body.postprocess(checker)
      },
      | lang::ControlFlow::If { .. }
      | lang::ControlFlow::For { .. }
      | lang::ControlFlow::Loop { .. } => todo!(),
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
      lang::Instruction::Block(block) => block.postprocess(checker),
      lang::Instruction::ControlFlow(ctrl_flow) => ctrl_flow.postprocess(checker),
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

    for argument in self.arguments.borrow_mut().inner.iter_mut() {
      argument.postprocess(checker)?;
    };

    self.body.postprocess(checker)?;

    Ok(())
  }
}

impl Postprocess for lang::ExternFunction {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    self.return_ty.postprocess(checker)?;

    for argument in self.arguments.borrow_mut().inner.iter_mut() {
      argument.postprocess(checker)?;
    };

    Ok(())
  }
}

impl Postprocess for lang::GenericConstraint {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    match self {
      lang::GenericConstraint::Extends { lhs, rhs, .. } => {
        lhs.postprocess(checker)?;
        rhs.postprocess(checker)?;
      },
    };

    Ok(())
  }
}

impl Postprocess for lang::GenericConstraints {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    for constraint in self.0.iter_mut() {
      constraint.postprocess(checker)?;
    };

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
        for member in members.borrow_mut().iter_mut() {
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
      lang::Type::Generic { constraints, .. } => constraints.postprocess(checker),
    }
  }
}

impl Postprocess for lang::TypeCell {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    self.borrow_mut().postprocess(checker)
  }
}

impl Postprocess for lang::Struct {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    for ty in self.members.borrow_mut().iter_mut() {
      ty.borrow_mut().postprocess(checker)?;
    };

    Ok(())
  }
}

impl Postprocess for DomainMember {
  fn postprocess(&mut self, checker: &mut TypeChecker) -> Result<(), TypeCheckerError> {
    if let Some(ref mut template_scope) = self.template_scope {
      for (_, ty) in template_scope.borrow_mut().iter_mut() {
        ty.postprocess(checker)?;
      };
    };

    match &mut self.kind {
      DomainMemberKind::Domain(domain) => domain.postprocess(checker),
      DomainMemberKind::Function(function) => function.postprocess(checker),
      DomainMemberKind::Type(r#type) => r#type.postprocess(checker),
      DomainMemberKind::ExternFunction(r#extern) => r#extern.postprocess(checker),
      DomainMemberKind::Struct(r#struct) => r#struct.postprocess(checker),
      // _ => todo!("{self:?}"),
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
