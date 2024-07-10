mod type_of;
mod coerce;
mod impls;
mod type_domain;

use std::collections::HashMap;

use crate::typechecker::{
  Domain,
  DomainMember,
  Program,
  TypeCheckerError,
  Handle,
};

use crate::typechecker::lang::{
  Function,
  Instruction,
  Variable,
  Type,
  TypeCell,
};

use type_domain::TypeDomain;
use coerce::Coerce;

pub(crate) use type_of::TypeOf;

#[allow(unused)]
pub(super) struct TypeChecker {
  // pub(super) program: Program,
  types: HashMap<Handle, TypeDomain>,
}

pub(super) trait Check {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError>;
}

impl TypeChecker {
  pub(super) fn new(program: &Program) -> Self {
    Self {
      types: TypeDomain::make_program_type_domain(program),
    }
  }
}
