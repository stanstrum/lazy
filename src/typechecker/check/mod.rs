mod type_of;
mod coerce;
mod impls;
mod type_domain;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::tokenizer::Span;
use crate::typechecker::{
  Domain,
  DomainMemberKind,
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
  // pub(super) program: Program,z
  types: HashMap<Handle, TypeDomain>,
  template_stack: Vec<Rc<RefCell<Vec<(String, TypeCell)>>>>,
}

pub(super) trait Check {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError>;
}

impl TypeChecker {
  pub(super) fn new(program: &Program) -> Self {
    Self {
      types: TypeDomain::make_program_type_domain(program),
      template_stack: vec![],
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

  fn resolve_type_reference_as_template(&self, reference: &DomainReference) -> Option<TypeCell> {
    if reference.inner.len() > 1 {
      todo!("memberof type reference not yet possible");
    };

    if reference.inner.is_empty() {
      panic!("empty reference");
    };

    let name = reference.inner.first().unwrap();

    for stack_item in self.template_stack.iter().rev() {
      for (template_name, ty) in stack_item.borrow().iter() {
        if template_name == name {
          return Some(ty.to_owned());
        };
      };
    };

    None
  }

  fn resolve_type_reference(&self, reference: &DomainReference, span: &Span) -> Result<TypeCell, TypeCheckerError> {
    if let Some(ty) = self.resolve_type_reference_as_template(reference) {
      return Ok(ty);
    };

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
