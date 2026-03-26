mod part;
mod pair;

use crate::lang::reference::TypeReference;
use crate::lang::ty::{Qualified, Type};
use crate::resolve::SpecialPair;

use super::*;

impl TypeOf for Type {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    match self {
      | Type::Intrinsic { .. }
      | Type::WeakInteger { .. }
      | Type::WeakFloat { .. }
      | Type::WeakString { .. }
      | Type::Weak { .. }
      | Type::ReferenceTo { .. }
      | Type::UnsizedArrayOf { .. }
      | Type::SizedArrayOf { .. }
      | Type::Unresolved { .. }
      => Ok(Some(self.clone())),
      // SPONGE
      // | Type::Unresolved { .. }
      //   => Ok(None),
      Type::Resolved { part, .. } => part.type_of(lazy),
      Type::Reference(reference) => reference.type_of(lazy),
    }
  }
}

impl Resolve for TypeReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let description = format!(line_dbg!("Resolve TypeReference: {}"), self.print(lazy));

    tasks.work(description, |tasks| match self {
      TypeReference::Part(type_part_reference) => {
        let ty = type_part_reference.rget_from(lazy);

        SpecialPair(self, ty).resolve(lazy, tasks)
      },
      TypeReference::ReturnTypeOf(function_reference) => {
        let ty = &function_reference.rget_from(lazy).header.ret_ty;

        SpecialPair(self, ty).resolve(lazy, tasks)
      },
      TypeReference::Alias(alias) => {
        alias.resolve(lazy, tasks)
      },
      TypeReference::Variable(v) => {
        let variable = v.rget_from(lazy);
        let ty = &variable.ty;

        SpecialPair(self, ty).resolve(lazy, tasks)
      },
      TypeReference::Expression(expr) => {
        expr.resolve(lazy, tasks)
      },
      TypeReference::Block(block) => {
        block.resolve(lazy, tasks)
      },
    })
  }
}

impl Coerce for TypeReference {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let Some(ty) = self.type_of(lazy)? else {
      dbg!(self.rget_from(lazy));

      todo!()
    };

    SpecialPair(self, &ty).coerce(lazy, other, tasks)
  }
}

pub(super) fn verify_typeof(lazy: &Lazy, ty: &(impl TypeOf + GetSpan + Pretty<Out = String>)) -> Result<()> {
  let Some(ty) = ty.type_of(lazy)? else {
    return Err(Box::new(Error::UnresolvedInVerify {
      what: ty.print(lazy),
      span: ty.get_span(lazy),
    }));
  };

  verify_type(lazy, &ty)
}

pub(super) fn verify_type(lazy: &Lazy, ty: &Type) -> Result<()> {
  match ty {
    Type::Reference(type_reference) => verify_typeof(lazy, type_reference),

    | Type::Resolved { part: ty, .. }
    | Type::ReferenceTo { ty, .. }
    | Type::UnsizedArrayOf { ty, .. }
    | Type::SizedArrayOf { ty, .. } => verify_type(lazy, ty.rget_from(lazy)),

    | &Type::Unresolved { qualified: Qualified { span, .. }, .. }
    | &Type::WeakInteger { span, .. }
    | &Type::WeakFloat { span, .. }
    | &Type::WeakString { span, .. }
    | &Type::Weak { span, .. }
      => Err(Box::new(Error::UnresolvedInVerify {
        what: ty.print(lazy),
        span,
      })),
    Type::Intrinsic { .. } => Ok(()),
  }
}
