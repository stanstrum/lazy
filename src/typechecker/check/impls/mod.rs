mod domain;

use crate::typechecker::lang::Block;
use crate::typechecker::lang::{
  self,
  intrinsics::Intrinsic,
  ExternFunction,
  Function,
  Instruction,
  Type,
  TypeCell,
};

use crate::typechecker::check::{
  Check,
  Coerce,
  CoerceCell,
  Extends,
  TypeChecker,
  TypeCheckerError,
  TypeOf,
  Variable,
};

impl Check for Variable {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    self.ty.check(checker)
  }
}

impl Check for lang::Value {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    match self {
      lang::Value::Variable(variable) => variable.get().check(checker),
      lang::Value::Instruction(instruction) => instruction.check(checker),
      lang::Value::Literal { ty, .. } => ty.check(checker),
    }
  }
}

impl Check for Instruction {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    Ok({
      match self {
        Instruction::Assign { dest, value, .. } => {
          match (dest.type_of(), value.type_of()) {
            (None, Some(rhs)) => {
              dest.coerce(checker, &rhs)?
            },
            (Some(lhs), None) => {
              value.coerce(checker, &lhs)?
            },
            _ => false,
          }
        },
        Instruction::Call { .. } => todo!(),
        Instruction::Return { value, to, span } => {
          if let Some(value) = value {
            let check = value.check(checker)?;
            let coerce = value.coerce_cell(checker, to)?;

            check || coerce
          } else {
            to.borrow().assert_extends(&Type::Intrinsic {
              kind: Intrinsic::Void,
              span: span.to_owned(),
            })?;

            false
          }
        },
        Instruction::Block(block) => block.check(checker)?,
        Instruction::Value(value) => value.check(checker)?,
      }
    })
  }
}

impl Check for Block {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    let mut did_work = false;

    for variable in self.variables.borrow_mut().inner.iter_mut() {
      did_work |= variable.check(checker)?;
    };

    for instruction in self.body.iter_mut() {
      did_work |= instruction.check(checker)?;
    };

    Ok(did_work)
  }
}

impl Check for Function {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    let mut did_work = false;

    for argument in self.arguments.inner.iter_mut() {
      did_work |= argument.check(checker)?;
    };

    did_work |= self.return_ty.check(checker)?;

    did_work |= self.body.check(checker)?;

    Ok(did_work)
  }
}

impl Check for ExternFunction {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    let mut did_work = false;

    for argument in self.arguments.inner.iter_mut() {
      did_work |= argument.check(checker)?;
    };

    did_work |= self.return_ty.check(checker)?;

    Ok(did_work)
  }
}

impl Check for TypeCell {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    self.borrow_mut().check(checker)
  }
}

impl Check for Type {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    let mut did_work = false;

    did_work |= match self {
      | Type::FuzzyInteger { .. }
      | Type::FuzzyString { .. } => false,
      Type::Unresolved { implied, .. } if *implied => todo!(),
      Type::Unresolved { template, .. } if template.is_some() => todo!(),
      Type::Unresolved { reference, span, .. } => {
        if let Some(ty) = checker.resolve_type_reference(reference, span)?.type_of() {
          *self = ty;

          true
        } else {
          false
        }
      },
      | Type::UnsizedArrayOf { ty, .. }
      | Type::SizedArrayOf { ty, .. }
      | Type::ReferenceTo { ty, .. }
      | Type::Shared(ty) => ty.check(checker)?,
      | Type::Function { args, return_ty, .. } => {
        for arg in args.iter_mut() {
          if arg.check(checker)? {
            did_work = true;
          };
        };

        return_ty.check(checker)?
      },
      Type::Struct { members: tys, .. } => {
        for ty in tys.iter_mut() {
          if ty.check(checker)? {
            did_work = true;
          };
        };

        false
      },
      | Type::Intrinsic { .. }
      | Type::Unknown { .. } => false,
    };

    Ok(did_work)
  }
}
