use std::collections::HashMap;

use crate::typechecker::{
  Domain,
  DomainMember,
  Program,
  Handle,
  lang::Type,
  lang::TypeCell,
};

pub(super) enum TypeDomainMember {
  Domain(TypeDomain),
  Type(TypeCell),
}

pub(super) struct TypeDomain(HashMap<String, TypeDomainMember>);

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

  pub(super) fn make_program_type_domain(program: &Program) -> HashMap<Handle, TypeDomain> {
    program.inner.iter()
      .map(
        |(handle, domain)|
          (*handle, Self::get_types_from_domain(domain))
      )
      .collect()
  }
}
