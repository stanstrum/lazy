use std::collections::HashMap;

use crate::typechecker::{
  Domain,
  DomainMemberKind,
  Program,
  Handle,
  lang::Type,
  lang::TypeCell,
};

use crate::tokenizer::GetSpan;

pub(super) enum TypeDomainMember {
  Domain(TypeDomain),
  Type(TypeCell),
}

pub(super) struct TypeDomain(pub(super) HashMap<String, TypeDomainMember>);

impl TypeDomain {
  fn get_types_from_domain(domain: &Domain) -> Self {
    Self(
      domain.inner.iter()
        .map(
          |(name, member)|
            match &member.kind {
              DomainMemberKind::Domain(domain) => (
                name.to_owned(),
                TypeDomainMember::Domain(
                  Self::get_types_from_domain(domain)
                )
              ),
              DomainMemberKind::Function(func) => (
                name.to_owned(),
                TypeDomainMember::Type(Type::Function {
                  args: func.arguments.borrow().inner.iter().map(|variable| variable.ty.to_owned()).collect(),
                  return_ty: func.return_ty.to_owned(),
                  span: func.get_span(),
                }.into())
              ),
              DomainMemberKind::Type(ty) => (
                name.to_owned(),
                TypeDomainMember::Type(ty.to_owned())
              ),
              DomainMemberKind::ExternFunction(r#extern) => (
                name.to_owned(),
                TypeDomainMember::Type(Type::Function {
                  args: r#extern.arguments.borrow().inner.iter().map(|variable| variable.ty.to_owned()).collect(),
                  return_ty: r#extern.return_ty.to_owned(),
                  span: r#extern.get_span(),
                }.into())
              ),
              DomainMemberKind::Struct(r#struct) => (
                name.to_owned(),
                TypeDomainMember::Type(Type::Struct {
                  members: r#struct.members.to_owned(),
                  span: r#struct.get_span(),
                }.into())
              )
            }
        )
        .collect()
    )
  }

  pub(super) fn make_program_type_domain(program: &Program) -> HashMap<Handle, TypeDomain> {
    program.inner.iter()
      .map(
        |(handle, data)|
          (*handle, Self::get_types_from_domain(&data.domain))
      )
      .collect()
  }
}
