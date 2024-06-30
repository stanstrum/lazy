use crate::typechecker::{
  TypeChecker,
  TypeCheckerError,
  lang,
};

pub(super) trait Coerce {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &lang::Type) -> Result<bool, TypeCheckerError>;
}

impl Coerce for lang::Value {
  fn coerce(&mut self, _checker: &mut TypeChecker, _to: &lang::Type) -> Result<bool, TypeCheckerError> {
    todo!()
  }
}
