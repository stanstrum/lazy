mod domain;

use crate::typechecker::check::{
  TypeChecker,
  TypeCheckerError,
  Check,
  TypeOf,
  Coerce,
  Variable,
  Instruction,
  Function,
  Type,
  TypeCell,
};

impl Check for Variable {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    self.ty.check(checker)
  }
}

impl Check for Instruction {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    Ok({
      match self {
        Instruction::Assign { dest, value } => {
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
        Instruction::Literal(_) => todo!(),
        Instruction::Return(..) => {
          dbg!("nothing done for return");

          false
        },
      }
    })
  }
}

impl Check for Function {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    for argument in self.arguments.inner.borrow_mut().iter_mut() {
      if argument.check(checker)? {
        return Ok(true);
      };
    };

    if self.return_ty.check(checker)? {
      return Ok(true);
    };

    for instruction in self.body.body.iter_mut() {
      if instruction.check(checker)? {
        return Ok(true);
      };
    };

    Ok(false)
  }
}

impl Check for TypeCell {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    self.borrow_mut().check(checker)
  }
}

impl Check for Type {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    Ok({
      match self {
        | Type::FuzzyInteger { .. }
        | Type::FuzzyString { .. }
        | Type::Unresolved { .. } => todo!(),
        | Type::UnsizedArrayOf { ty, .. }
        | Type::SizedArrayOf { ty, .. }
        | Type::ReferenceTo { ty, .. }
        | Type::Shared(ty) => ty.check(checker)?,
        | Type::Function { args, return_ty, .. } => {
          for arg in args.iter_mut() {
            if arg.check(checker)? {
              return Ok(true);
            };
          };

          return_ty.check(checker)?
        },
        Type::Struct { members: tys, .. } => {
          for ty in tys.iter_mut() {
            if ty.check(checker)? {
              return Ok(true);
            };
          };

          false
        },
        | Type::Intrinsic(_)
        | Type::Unknown => false,
      }
    })
  }
}
