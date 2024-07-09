use crate::typechecker::check::{
  TypeChecker,
  TypeCheckerError,
  Check,
  Domain,
  DomainMember,
  Program,
};

impl Check for Domain {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    for member in self.inner.values_mut() {
      if match member {
        DomainMember::Domain(domain) => domain.check(checker)?,
        DomainMember::Function(func) => func.check(checker)?,
        DomainMember::Type(ty) => ty.check(checker)?,
      } {
        return Ok(true);
      };
    };

    Ok(false)
  }
}

impl Check for Program {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    for data in self.inner.values_mut() {
      if data.domain.check(checker)? {
        return Ok(true);
      };
    };

    Ok(false)
  }
}
