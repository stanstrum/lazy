use crate::typechecker::{
  TypeChecker,
  TypeCheckerError,
};

use crate::typechecker::lang::{
  Instruction,
  VariableReference,
  Value,
  Type,
  TypeCell,
};

pub(super) trait Coerce {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError>;
}

pub(crate) trait Extends where Self: std::fmt::Debug + crate::typechecker::lang::pretty_print::PrettyPrint {
  fn extends(&self, other: &Self) -> bool;
  fn assert_extends(&self, other: &Self) -> Result<(), TypeCheckerError> {
    if !self.extends(other) {
      todo!("not extends error here: {} and {}", self.pretty_print(), other.pretty_print());
    };

    Ok(())
  }
}

impl Extends for Type {
  fn extends(&self, other: &Self) -> bool {
    match (self, &other) {
      (Type::Unknown, _) => true,
      _ => false,
    }
  }
}

impl Extends for TypeCell {
  fn extends(&self, other: &Self) -> bool {
    self.borrow().extends(&*other.borrow())
  }
}

impl Coerce for TypeCell {
  fn coerce(&mut self, _checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError> {
    let inner = &mut *self.borrow_mut();

    inner.assert_extends(to)?;

    match (&inner, to) {
      (Type::Unknown, _) => {
        *inner = to.to_owned();

        Ok(true)
      },
      _ => panic!("not coercible:\n- {inner:#?}\n\n- {to:#?}")
    }
  }
}

impl Coerce for VariableReference {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError> {
    self.get().ty.coerce(checker, to)
  }
}

impl Coerce for Instruction {
  fn coerce(&mut self, _checker: &mut TypeChecker, _to: &Type) -> Result<bool, TypeCheckerError> {
    todo!()
  }
}

impl Coerce for Value {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError> {
    match self {
      Value::Variable(var) => var.coerce(checker, to),
      Value::Instruction(inst) => inst.coerce(checker, to),
    }
  }
}
