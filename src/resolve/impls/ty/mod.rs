mod part;
mod pair;

use crate::lang::reference::TypeReference;
use crate::lang::ty::{Qualified, Type};
use crate::resolve::tasks::OverwriteTypeReference;
use crate::resolve::{TypePair, TypePairModifier};

use super::*;

trait DereferenceType {
  fn dereference(&self, lazy: &Lazy, r#mut: bool) -> Result<Option<TypePair>>;
}

impl<T: TypeOf> DereferenceType for T {
  fn dereference(&self, lazy: &Lazy, r#mut: bool) -> Result<Option<TypePair>> {
    let Some(ty) = self.type_of(lazy)? else {
      return Ok(None);
    };

    match ty {
      Type::Reference(type_reference) => type_reference.dereference(lazy, r#mut),
      Type::Resolved { part, .. } => dbg!(part.dereference(lazy, r#mut)),
      Type::Unresolved { .. } => Ok(None),
      Type::Intrinsic { .. } => Ok(None),
      Type::WeakInteger { .. } => Ok(None),
      Type::WeakFloat { .. } => Ok(None),
      Type::WeakString { dereferenced: true, .. } => Ok(None),
      Type::WeakString { kind, characters, span, .. } => Ok({
        let mut reference = self.reference(lazy).expect("please please please");
        let ty = Type::WeakString {
          dereferenced: true,
          kind,
          characters,
          span,
        };

        reference.modifiers.push(TypePairModifier::Dereference);

        Some(TypePair {
          reference: reference.reference,
          modifiers: reference.modifiers,
          ty,
        })
      }),
      Type::Weak { .. } => Ok(None),
      Type::ReferenceTo { ty, r#mut: reference_mut, .. } => Ok({
        #[allow(clippy::nonminimal_bool)]
        (!(reference_mut && !r#mut)).then(|| {
          let mut reference = self.reference(lazy).expect("please please please");
          reference.modifiers.push(TypePairModifier::Dereference);

          TypePair {
            reference: reference.reference,
            modifiers: reference.modifiers,
            ty: Type::Reference(TypeReference::Part(ty)),
          }
        })
      }),
      Type::UnsizedArrayOf { .. } => Ok(None),
      Type::SizedArrayOf { .. } => Ok(None),
    }
  }
}

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

  fn reference(&self, lazy: &Lazy) -> Option<OverwriteTypeReference> {
    match dbg!(self) {
      &Type::Reference(type_reference) => Some(type_reference.into()),
      &Type::Resolved { part, .. } => Some(TypeReference::Part(part).into()),
      Type::Unresolved { .. } => todo!(),
      Type::Intrinsic { .. } => todo!(),
      Type::WeakInteger { .. } => todo!(),
      Type::WeakFloat { .. } => todo!(),
      Type::WeakString { .. } => todo!(),
      Type::Weak { .. } => todo!(),
      Type::ReferenceTo { .. } => todo!(),
      Type::UnsizedArrayOf { .. } => todo!(),
      Type::SizedArrayOf { .. } => todo!(),
    }
  }
}

impl TypeOf for TypeReference {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    self.rget_from(lazy).type_of(lazy)
  }

  fn reference(&self, lazy: &Lazy) -> Option<OverwriteTypeReference> {
    Some((*self).into())
  }
}

impl Resolve for TypeReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let description = format!(line_dbg!("Resolve TypeReference: {}"), self.print(lazy));

    tasks.work(description, |tasks| match self {
      TypeReference::Part(type_part_reference) => {
        let ty = type_part_reference.rget_from(lazy);

        TypePair::new(*self, ty.clone()).resolve(lazy, tasks)
      },
      TypeReference::ReturnTypeOf(function_reference) => {
        let ty = &function_reference.rget_from(lazy).header.ret_ty;

        TypePair::new(*self, ty.clone()).resolve(lazy, tasks)
      },
      TypeReference::Alias(alias) => {
        let ty = &alias.rget_from(lazy).ty;

        TypePair::new(*self, ty.clone()).resolve(lazy, tasks)
      },
      TypeReference::Variable(v) => {
        let variable = v.rget_from(lazy);
        let ty = &variable.ty;

        TypePair::new(*self, ty.clone()).resolve(lazy, tasks)
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
    // let Some(ty) = self.type_of(lazy)? else {
    //   dbg!(self.rget_from(lazy));

    //   todo!()
    // };

    let overwrite: OverwriteTypeReference = (*self).into();
    overwrite.coerce(lazy, other, tasks)
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
