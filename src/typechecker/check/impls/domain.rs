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
    let mut did_work = false;

    for member in self.inner.values_mut() {
      did_work |= match member {
        DomainMember::Domain(domain) => domain.check(checker)?,
        DomainMember::Function(func) => func.check(checker)?,
        DomainMember::Type(ty) => ty.check(checker)?,
        DomainMember::ExternFunction(r#extern) => r#extern.check(checker)?,
      };
    };

    Ok(did_work)
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
