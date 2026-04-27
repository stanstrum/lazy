mod part;
pub mod pair;

use lang::Compiler;
use lang::ty::{Qualified, Type, TypePair};
use lang::reference::TypeReference;

use super::*;

trait DereferenceType<C: Compiler> {
  fn dereference(&self, store: &C::Store<'_>, r#mut: bool) -> Result<C, Option<TypePair<C>>>;
}

impl<C: Compiler, T: TypeOf<C>> DereferenceType<C> for T {
  fn dereference(&self, store: &C::Store<'_>, r#mut: bool) -> Result<C, Option<TypePair<C>>> {
    let Some(ty) = self.type_of(store) else {
      return Ok(None);
    };

    match ty {
      Type::Reference(type_reference) => type_reference.dereference(store, r#mut),
      Type::Resolved { part, .. } => dbg!(part.dereference(store, r#mut)),
      Type::Unresolved { .. } => Ok(None),
      Type::Intrinsic { .. } => Ok(None),
      Type::WeakInteger { .. } => Ok(None),
      Type::WeakFloat { .. } => Ok(None),
      Type::WeakString { dereferenced: true, .. } => Ok(None),
      Type::WeakString { kind, characters, span, .. } => Ok({
        todo!()
        // let mut reference = self.reference(store).expect("please please please");
        // let ty = Type::WeakString {
        //   dereferenced: true,
        //   kind,
        //   characters,
        //   span,
        // };

        // reference.modifiers.push(TypePairModifier::Dereference);

        // Some(TypePair { overwrite: reference, ty, })
      }),
      Type::Weak { .. } => Ok(None),
      Type::ReferenceTo { ty, r#mut: reference_mut, .. } => Ok({
        #[allow(clippy::nonminimal_bool)]
        (!(reference_mut && !r#mut)).then(|| {
          todo!()
          // let mut reference = self.reference(store).expect("please please please");
          // reference.modifiers.push(TypePairModifier::Dereference);

          // TypePair {
          //   overwrite: reference,
          //   ty: Type::Reference(TypeReference::Part(ty)),
          // }
        })
      }),
      Type::UnsizedArrayOf { .. } => Ok(None),
      Type::SizedArrayOf { .. } => Ok(None),
      Type::Struct { .. } => Ok(None),
    }
  }
}

impl<C: Compiler + 'static> Resolve<C> for TypeReference<C> {
  fn resolve(&self, store: &C::Store<'_>, tasks: &mut Tasks<C>) -> Result<C> {
    let description = format!(line_dbg!("Resolve TypeReference: {}"), self.print(store));

    tasks.work(description, |tasks| match self {
      TypeReference::Part(type_part_reference) => {
        let ty = type_part_reference.rget_from(store);

        TypePair::new(*self, ty.clone()).resolve(store, tasks)
      },
      TypeReference::ReturnTypeOf(function_reference) => {
        let ty = &function_reference.rget_from(store).header.ret_ty;

        TypePair::new(*self, ty.clone()).resolve(store, tasks)
      },
      TypeReference::StructMember(struct_reference, id) => {
        let ty = &struct_reference.rget_from(store).members.get(*id).unwrap().ty;

        TypePair::new(*self, ty.clone()).resolve(store, tasks)
      },
      TypeReference::Alias(alias) => {
        let ty = &alias.rget_from(store).ty;

        TypePair::new(*self, ty.clone()).resolve(store, tasks)
      },
      TypeReference::Variable(v) => {
        let variable = v.rget_from(store);
        let ty = &variable.ty;

        TypePair::new(*self, ty.clone()).resolve(store, tasks)
      },
      TypeReference::Expression(expr) => {
        expr.resolve(store, tasks)
      },
      TypeReference::Block(block) => {
        block.resolve(store, tasks)
      },
    })
  }
}

impl<C: Compiler + 'static> Coerce<C> for TypeReference<C> {
  fn coerce(&self, store: &C::Store<'_>, other: &impl TypeOf<C>, tasks: &mut Tasks<C>) -> Result<C> {
    // let Some(ty) = self.type_of(lazy)? else {
    //   dbg!(self.rget_from(lazy));

    //   todo!()
    // };

    todo!()
    // let overwrite: OverwriteTypeReference<C> = (*self).into();
    // overwrite.coerce(store, other, tasks)
  }
}

pub(super) fn verify_typeof<C: Compiler + 'static>(
  store: &C::Store<'_>,
  ty: &(impl TypeOf<C> + Pretty<C, Out = String>),
  tasks: &mut Tasks<C>,
) -> Result<C> {
  let description = format!(line_dbg!("Verify type via TypeOf: {}"), ty.print(store));

  tasks.work(description, |tasks| {
    let Some(ty) = ty.type_of(store) else {
      return tasks.seed_error(ResolveErrorBase::UnresolvedInVerify {
        what: ty.print(store),
        span: ty.get_span(store),
      });
    };

    verify_type(store, &ty, tasks)
  })
}

pub(super) fn default_types_of_type<C: Compiler + 'static>(store: &mut C::Store<'_>, reference: &TypeReference<C>, tasks: &mut Tasks<C>) -> Result<C> {
  let ty = reference.type_of(store).expect("to get a type");
  let pair = TypePair::new(*reference, ty);

  ty::pair::default_types_of_type_pair(store, &pair, tasks)
}

pub(super) fn verify_type<C: Compiler + 'static>(store: &C::Store<'_>, ty: &Type<C>, tasks: &mut Tasks<C>) -> Result<C> {
  let description = format!(line_dbg!("Verify type {}"), ty.print(store));

  tasks.work(description, |tasks| match ty {
    Type::Reference(type_reference) => verify_typeof(store, type_reference, tasks),

    | Type::Resolved { part: ty, .. }
    | Type::ReferenceTo { ty, .. }
    | Type::UnsizedArrayOf { ty, .. }
    | Type::SizedArrayOf { ty, .. } => verify_type(store, ty.rget_from(store), tasks),

    | &Type::Unresolved { qualified: Qualified { span, .. }, .. }
    | &Type::WeakInteger { span, .. }
    | &Type::WeakFloat { span, .. }
    | &Type::WeakString { span, .. }
    | &Type::Weak { span, .. }
      => tasks.seed_error(ResolveErrorBase::UnresolvedInVerify {
        what: ty.print(store),
        span,
      }),
    Type::Intrinsic { .. } => Ok(()),
    Type::Struct { prototype } => structure::verify_struct(store, prototype, tasks),
  })
}
