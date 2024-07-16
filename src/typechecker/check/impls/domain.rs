use crate::typechecker::check::{
  TypeChecker,
  TypeCheckerError,
  Check,
  Domain,
  DomainMemberKind,
  Program,
};

impl Check for Domain {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    let mut did_work = false;

    for member in self.inner.values_mut() {
      if let Some(template_scope) = member.template_scope.as_ref() {
        checker.template_stack.push(dbg!(template_scope.to_owned()));
      };

      did_work |= match &mut member.kind {
        DomainMemberKind::Domain(domain) => domain.check(checker)?,
        DomainMemberKind::Function(func) => func.check(checker)?,
        DomainMemberKind::Type(ty) => ty.check(checker)?,
        DomainMemberKind::ExternFunction(r#extern) => r#extern.check(checker)?,
        DomainMemberKind::Struct(r#struct) => r#struct.check(checker)?,
      };

      if member.template_scope.is_some() {
        checker.template_stack.pop();
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
