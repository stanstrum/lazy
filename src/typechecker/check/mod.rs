mod type_of;
mod coerce;
mod impls;
mod type_domain;

use std::collections::HashMap;

use crate::tokenizer::Span;
use crate::typechecker::{
  Domain,
  DomainMember,
  Handle,
  Program,
  TypeCheckerError,
};

use crate::typechecker::lang::{
  Variable,
  Type,
  TypeCell,
};

use type_domain::{
  TypeDomain,
  TypeDomainMember,
};

pub(in crate::typechecker) use coerce::{
  Coerce,
  CoerceCell,
  Extends,
};

pub(crate) use type_of::TypeOf;

use super::lang::pretty_print::PrettyPrint;
use super::postprocess::Postprocess;
use super::{DomainReference, InvalidSnafu};

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

  pub(super) fn check_until_done(&mut self, program: &mut Program) -> Result<(), TypeCheckerError> {
    for pass in 1.. {
      println!("Pass {pass} ...");

      if !dbg!(program.check(self)?) {
        break;
      };
    };

    Ok(())
  }

  pub(super) fn postprocess(&mut self, program: &mut Program) -> Result<(), TypeCheckerError> {
    program.postprocess(self)
  }

  fn resolve_type_reference(&self, reference: &DomainReference, span: &Span) -> Result<TypeCell, TypeCheckerError> {
    let mut map = &self.types.get(&reference.handle).unwrap().0;

    let Some((last, parts)) = reference.inner.split_last() else {
      return InvalidSnafu {
        message: format!(
          "not found: {}",
          Type::Unresolved { implied: false, reference: reference.to_owned(), template: None, span: *span }.pretty_print(),
        ),
        span: *span,
      }.fail();
    };

    for part in parts {
      let Some(TypeDomainMember::Domain(next)) = map.get(part) else {
        return InvalidSnafu {
          message: format!(
            "not found: {}",
            part.to_owned(),
          ),
          span: *span,
        }.fail();
      };

      map = &next.0;
    };

    match map.get(last) {
      Some(TypeDomainMember::Type(ty)) => Ok(ty.to_owned()),
      _ => InvalidSnafu {
        message: format!("not found: {}", last.to_owned()),
        span: span.to_owned(),
      }.fail(),
    }
  }
}
