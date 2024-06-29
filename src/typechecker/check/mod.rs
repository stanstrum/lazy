use std::collections::HashMap;

use super::{
  Domain,
  DomainMember,
  Program,
  TypeCheckerError,
};

use super::lang::{
  Instruction, TypeCell, Value, Variable
};

use crate::compiler::Handle;
use crate::typechecker::lang::{
  Function,
  Type,
};

enum TypeDomainMember {
  Domain(TypeDomain),
  Type(TypeCell),
}

struct TypeDomain(HashMap<String, TypeDomainMember>);

impl TypeDomain {
  fn get_types_from_domain(domain: &Domain) -> Self {
    Self(
      domain.inner.iter()
        .filter_map(
          |(name, member)|
            match member {
              DomainMember::Domain(domain) => Some((
                name.to_owned(),
                TypeDomainMember::Domain(
                  Self::get_types_from_domain(domain)
                )
              )),
              DomainMember::Function(func) => Some((
                name.to_owned(),
                TypeDomainMember::Type(Type::Function {
                  args: func.arguments.inner.borrow().iter().map(|variable| variable.ty.to_owned()).collect(),
                  return_ty: func.return_ty.to_owned(),
                }.into())
              )),
              DomainMember::Type(ty) => Some((
                name.to_owned(),
                TypeDomainMember::Type(ty.to_owned())
              )),
            }
        )
        .collect()
    )
  }

  fn make_program_type_domain(program: &Program) -> HashMap<Handle, TypeDomain> {
    program.inner.iter()
      .map(
        |(handle, domain)|
          (*handle, Self::get_types_from_domain(domain))
      )
      .collect()
  }
}

pub(super) struct TypeChecker {
  // pub(super) program: Program,
  types: HashMap<Handle, TypeDomain>,
}

impl TypeChecker {
  pub(super) fn new(program: &Program) -> Self {
    Self {
      types: TypeDomain::make_program_type_domain(&program),
    }
  }
}

trait Check {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError>;
}

trait TypeOf {
  fn type_of(&self) -> Option<Type>;

  // fn type_of_expect(&self) -> Result<Type, TypeCheckerError> {
  //   let Some(ty) = self.type_of() else {
  //     todo!()
  //   };

  //   Ok(ty)
  // }
}

trait Coerce {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError>;
}

impl Coerce for Value {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError> {
    todo!()
  }
}

impl TypeOf for Type {
  fn type_of(&self) -> Option<Type> {
    match self {
      Type::Intrinsic(_) => todo!(),
      Type::Unresolved { implied, reference, template } => todo!(),
      Type::UnsizedArrayOf(_) => todo!(),
      Type::SizedArrayOf { count, ty } => todo!(),
      Type::ReferenceTo { r#mut, ty } => todo!(),
      Type::Shared(_) => todo!(),
      Type::Function { args, return_ty } => todo!(),
      Type::Unknown => todo!(),
    }
  }
}

impl TypeOf for TypeCell {
  fn type_of(&self) -> Option<Type> {
    self.borrow().type_of()
  }
}

impl TypeOf for Value {
  fn type_of(&self) -> Option<Type> {
    match self {
      Value::Variable(var) => var.get().ty.type_of(),
      Value::Instruction(_) => todo!(),
    }
  }
}

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
        Instruction::Call { func, args } => todo!(),
        Instruction::Literal(_) => todo!(),
        Instruction::Return(_) => todo!(),
      }
    })
  }
}

impl Check for Function {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    for argument in self.arguments.inner.try_borrow_mut().unwrap().iter_mut() {
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
    self.try_borrow_mut().unwrap().check(checker)
  }
}

impl Check for Type {
  fn check(&mut self, checker: &mut TypeChecker) -> Result<bool, TypeCheckerError> {
    Ok({
      match self {
        Type::Unresolved { .. } => todo!(),
        | Type::UnsizedArrayOf(ty)
        | Type::SizedArrayOf { ty, .. }
        | Type::ReferenceTo { ty, .. }
        | Type::Shared(ty) => ty.check(checker)?,
        | Type::Function { args, return_ty } => {
          for arg in args.iter_mut() {
            if arg.check(checker)? {
              return Ok(true);
            };
          };

          return_ty.check(checker)?
        },
        | Type::Intrinsic(_)
        | Type::Unknown => false,
      }
    })
  }
}

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
    for domain in self.inner.values_mut() {
      if domain.check(checker)? {
        return Ok(true);
      };
    };

    Ok(false)
  }
}

impl TypeChecker {
  pub(super) fn check(&mut self, program: &mut Program) -> Result<bool, TypeCheckerError> {
    program.check(self)
  }
}
