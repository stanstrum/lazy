use crate::lang::ty::Type;
use crate::lang::Lazy;
use crate::lang::reference::{Reference, Store, TypePartReference, TypeReference};
use crate::resolve::coerce::{SpecialPair, TypePair};
use crate::resolve::tasks::ResolveType;
use crate::resolve::ty::unknown::resolve_qualified_to_type;

use super::{Result, Error, Tasks, Resolve};

mod unknown;

// #[derive(Debug)]
// pub struct SpecialPair<'a>(pub &'a TypeReference, pub &'a Type);

impl Resolve for TypePartReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let reference = TypeReference::Part(*self);
    let ty = self.rget_from(lazy);

    SpecialPair(&reference, ty).resolve(lazy, tasks)
  }
}

impl Resolve for TypeReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    match self {
      TypeReference::Part(type_part_reference) => {
        let ty = type_part_reference.rget_from(lazy);

        SpecialPair(self, ty).resolve(lazy, tasks)
      },
      TypeReference::ReturnTypeOf(function_reference) => {
        let ty = &function_reference.rget_from(lazy).header.ret_ty;

        SpecialPair(self, ty).resolve(lazy, tasks)
      },
      TypeReference::Alias(_) => todo!(),
      TypeReference::ArgumentOf(function_reference, index) => {
        let function = function_reference.rget_from(lazy);
        let variable = function.header.arguments.get(*index).unwrap();
        let ty = &variable.ty;

        SpecialPair(self, ty).resolve(lazy, tasks)
      },
      TypeReference::Expression(_) => todo!(),
      TypeReference::Block(_) => todo!(),
    }
  }
}

impl<'a, 'b> Resolve for TypePair<'a, 'b> {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let SpecialPair(reference, ty) = self;

    match ty {
      Type::Unresolved { module, qualified } => {
        if let Some(ty) = resolve_qualified_to_type(lazy, *module, qualified)? {
          tasks.push(ResolveType {
            dest: **reference,
            value: ty,
          });
        };

        Ok(())
      },
      Type::Intrinsic { .. } => {
        // do nothing ...
        Ok(())
      },
      Type::WeakInteger { .. } => todo!(),
      Type::WeakFloat { .. } => todo!(),
      Type::WeakString { .. } => todo!(),
      Type::Weak { .. } => todo!(),
      | Type::ReferenceTo { ty, .. }
      | Type::UnsizedArrayOf { ty, .. }
      | Type::SizedArrayOf { ty, .. }
      | Type::Resolved { part: ty, .. }
        => ty.resolve(lazy, tasks),
      // Type::Expression(expression_reference) => todo!(),
      | Type::Reference(reference) => {
        let ty = reference.rget_from(lazy);
        SpecialPair(reference, ty).resolve(lazy, tasks)
      },
    }
  }
}
