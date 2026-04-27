pub mod unknown;

use lang::Compiler;
use lang::token::StringKind;
use lang::module::AddTypePart;
use lang::intrinsic::Intrinsic;
use lang::ty::{QualifiedSearchSpace, Type};

use super::*;

// impl DereferenceType for TypePair {
//   fn dereference(&self, lazy: &C::Store<'_>, r#mut: bool) -> Result<C, Option<Type>> {
//     let modifiers = self.modifiers.clone();
//     let reference = *self.pair.0;

//     clone.modifiers.push(TypePairModifier::Dereference);

//     Ok(Some(clone))
//   }
// }

impl<C: Compiler + 'static> Resolve<C> for Type<C> {
  fn resolve(&self, store: &C::Store<'_>, tasks: &mut Tasks<C>) -> Result<C> {
    let description = format!(line_dbg!("Resolve TypePair:\n- Ref.: {}\n- Type: {}"),
      self.reference.print(store),
      self.ty.print(store),
    );

    tasks.work(description, |tasks| {
      match &self.ty {
        TypeKind::Unresolved { module, qualified } => {
          if let Some(ty) = unknown::resolve_qualified_to_type(store, *module, qualified, tasks)? {
            todo!()
            // tasks.push(tasks::Subjugate {
            //   prerequisite: Box::new(tasks::OverwriteType {
            //     dest: self.clone().into(),
            //     src: ty,
            //   }),
            //   after: Box::new(tasks::ResolveAsTask::<C, TypeReference<C>>::new(self.overwrite.reference)),
            // }, line_dbg!("here"));
          };

          Ok(())
        },
        TypeKind::Intrinsic { .. } => {
          // do nothing ...
          Ok(())
        },
        | TypeKind::WeakInteger { .. }
        | TypeKind::WeakFloat { .. }
        | TypeKind::WeakString { .. }
        | TypeKind::Weak { .. } => {
          // do nothing ... can't resolve this
          Ok(())
        }
        | TypeKind::ReferenceTo { ty, .. }
        | TypeKind::UnsizedArrayOf { ty, .. }
        | TypeKind::SizedArrayOf { ty, .. }
        | TypeKind::Resolved { part: ty, .. }
          => ty.resolve(store, tasks),
        | TypeKind::Reference(reference) => {
          let ty = reference.rget_from(store).clone();

          let mut new_reference = self.clone();
          new_reference.ty = ty;

          new_reference.resolve(store, tasks)
        },
        TypeKind::Struct { prototype } => prototype.resolve(store, tasks),
      }
    })
  }
}

impl<C: Compiler + 'static> Coerce<C> for Type<C> {
  fn coerce(&self, store: &C::Store<'_>, other_ref: &impl TypeOf<C>, tasks: &mut Tasks<C>) -> Result<C> {
    let a = self.reference.print(store);
    let b = self.ty.print(store);
    let c = other_ref.type_of(store)
      .map(|x| x.print(store))
      .unwrap_or_else(|| "{none}".into());
    todo!();
    let d = /* format!("{:?}", &self.overwrite.modifiers); */ todo!();

    let description = format!(
      line_dbg!("Coerce TypePair\n- Reference: {}\n- Modifiers: {}\n- Type:      {}\n- Coerce w/: {}"),
      a, d, b, c,
    );

    tasks.work(description, |tasks| {
      // println!(line_dbg!("here:\n{}"), tasks.explain(2));

      let Some(other) = other_ref.type_of(store) else {
        return Ok(());
      };

      // println!(
      //   line_dbg!("TypePair({}) coerced by {}"),
      //   self.print(lazy),
      //   other.print(lazy),
      // );

      match (&self.ty, &other) {
        (TypeKind::Intrinsic { kind: kind_a, .. }, TypeKind::Intrinsic { kind: kind_b, .. })
          if kind_a == kind_b
        => {
          // do nothing
          Ok(())
        },
        (
          | TypeKind::WeakInteger { .. }
          | TypeKind::WeakFloat { .. },
          TypeKind::Intrinsic { kind, .. },
        ) if !matches!(kind, Intrinsic::Bool | Intrinsic::Void) => {
          todo!();
          // tasks.push(tasks::OverwriteType {
          //   dest: self.clone().into(),
          //   src: other,
          // }, line_dbg!("here"));

          Ok(())
        },
        (TypeKind::Intrinsic { kind, .. }, TypeKind::WeakInteger { .. }) if kind.is_integer() => {
          Ok(())
        },
        (TypeKind::Resolved { part, .. }, _) => {
          let ty = part.rget_from(store);
          let reference = TypeReference::Part(*part);

          Type::new(reference, ty.clone()).coerce(store, other_ref, tasks)
        },
        (TypeKind::Reference(reference), _) => {
          let ty = reference.rget_from(store);
          Type::new(*reference, ty.clone()).coerce(store, other_ref, tasks)
        },
        (_, TypeKind::Resolved { part, .. }) => {
          let ty = part.rget_from(store);
          let reference = TypeReference::Part(*part);
          let other_ref = Type::new(reference, ty.clone());
          self.coerce(store, &other_ref, tasks)
        },
        (_, TypeKind::Reference(reference)) => {
          let ty = reference.rget_from(store);
          let other_ref = Type::new(*reference, ty.clone());
          self.coerce(store, &other_ref, tasks)
        },
        // SPONGE: structs can be equivalent to one another without being the
        //         exact same ...
        (TypeKind::Struct { prototype: lhs }, TypeKind::Struct { prototype: rhs }) if lhs == rhs => {
          Ok(())
        },
        (TypeKind::Weak { .. }, _) => {
          todo!();

          // tasks.push(tasks::OverwriteType {
          //   dest: self.overwrite.clone(),
          //   src: other,
          // }, line_dbg!("here"));

          Ok(())
        },
        (_, TypeKind::Unresolved { .. })
          => {
          // An unresolved doesn't tell us much
          Ok(())
        },
        (TypeKind::Unresolved { module, qualified }, _) if qualified.is_implicit() => {
          let dest = /* self.clone().into() */ todo!();

          let mut qualified = qualified.clone();
          todo!();
          // qualified.implicit = QualifiedSearchSpace::Type(other_ref.reference(store).expect("god help me"));

          let src = TypeKind::Unresolved {
            module: *module,
            qualified,
          };

          todo!();
          // // RHS should be anything but another Unresolved.  Try to resolve
          // // an implicit
          // let task = /* tasks::Subjugate {
          //   prerequisite: Box::new(tasks::OverwriteType {
          //     dest,
          //     src,
          //   }),
          //   after: Box::new(tasks::ResolveAsTask::new(self.overwrite.reference)),
          // } */ todo!();

          // tasks.push(task, line_dbg!("here"));

          Ok(())
        },
        (TypeKind::Unresolved { .. }, _) => {
          // Can't just throw an error here.  A task could have yet to come
          // around and update this.  Let the resolver or verifier sort out
          // this mess.
          Ok(())
        },
        (
          TypeKind::SizedArrayOf { ty: ty_a, .. } | TypeKind::UnsizedArrayOf { ty: ty_a, .. },
          TypeKind::SizedArrayOf { ty: ty_b, .. } | TypeKind::UnsizedArrayOf { ty: ty_b, .. },
        ) => {
          ty_a.coerce(store, ty_b, tasks)?;
          ty_b.coerce(store, ty_a, tasks)?;

          Ok(())
        },
        (
          TypeKind::WeakString { kind: kind_a, characters: characters_a, dereferenced: dereferenced_a, .. },
          TypeKind::WeakString { kind: kind_b, characters: characters_b, dereferenced: dereferenced_b, .. },
        ) => {
          assert!(
            matches!(
              (kind_a, kind_b),
              | (StringKind::Wide, StringKind::Wide)
              | (StringKind::Byte, StringKind::Byte)
              | (StringKind::Byte, StringKind::C)
              | (StringKind::C, StringKind::Byte)
              | (StringKind::C, StringKind::C)
            )
          );

          assert!(characters_a == characters_b);
          assert!(dereferenced_a == dereferenced_b);

          Ok(())
        },
        | (
          TypeKind::SizedArrayOf { ty, size, .. },
          TypeKind::WeakString { kind, characters, dereferenced, span },
        )
        | (
          TypeKind::WeakString { kind, characters, dereferenced, span },
          TypeKind::SizedArrayOf { ty, size, .. },
        ) /* if size == characters */ => {
          assert!(dereferenced, "todo");
          assert!(size == characters, "throw error for weak string size mismatch");

          ty.coerce(store, &TypeKind::Intrinsic {
            kind: (*kind).into(),
            span: *span,
          }, tasks)?;

          Ok(())
        },
        | (
          TypeKind::UnsizedArrayOf { ty, .. },
          TypeKind::WeakString { kind, span, .. },
        )
        | (
          TypeKind::WeakString { kind, span, .. },
          TypeKind::UnsizedArrayOf { ty, .. },
        ) => {
          ty.coerce(store, &TypeKind::Intrinsic {
            kind: (*kind).into(),
            span: *span,
          }, tasks)
        },
        (a, b) => {
          // #[cfg(debug_assertions)] dbg!(a, b);

          if
            let Some(a) = self.dereference(store, false)? &&
            let Some(b) = other_ref.dereference(store, false)?
          {
            return a.coerce(store, &b, tasks);
          };

          tasks.seed_error(ResolveErrorBase::TypeMismatch {
            whence: line_dbg!(),
            a_print: a.print(store),
            a_span: a.get_span(store),
            b_print: b.print(store),
            b_span: b.get_span(store),
          })
        },
      }
    })
  }
}

pub(in crate::impls) fn default_types_of_type_pair<C: Compiler + 'static>(store: &mut C::Store<'_>, pair: &Type<C>, tasks: &mut Tasks<C>) -> Result<C> {
  match &pair.ty {
    TypeKind::Reference(ty) => default_types_of_type(store, ty, tasks),
    TypeKind::Resolved { part, .. } => default_types_of_type(
      store, &TypeReference::Part(*part), tasks,
    ),
    TypeKind::Unresolved { .. } => todo!(),
    TypeKind::Intrinsic { .. } => Ok(()),
    TypeKind::WeakInteger { span, .. } => {
      let src = TypeKind::Intrinsic {
        kind: Intrinsic::U32,
        span: *span,
      };

      todo!();
      // tasks.push(tasks::OverwriteType {
      //   dest: pair.to_owned().into(),
      //   src,
      // }, line_dbg!("here"));

      Ok(())
    },
    TypeKind::WeakFloat { .. } => todo!(),
    &TypeKind::WeakString { kind, characters, dereferenced, span } => {
      // dbg!(kind, characters, dereferenced);

      if dereferenced {
        todo!()
      };

      let src = /* {
        let parent_module = pair.overwrite.reference.parent_module(store) todo!();

        let element_intrinsic = kind.into();
        let element_part = Type::Intrinsic { kind: element_intrinsic, span };
        let element_reference = parent_module.add_type_part(element_part, store);

        let arr_of_element_part = Type::SizedArrayOf { ty: element_reference, size: characters, span };
        let arr_of_element_reference = parent_module.add_type_part(arr_of_element_part, store);

        Type::ReferenceTo { ty: arr_of_element_reference, r#mut: false, span }
      } */ todo!();

      // tasks.push(tasks::OverwriteType {
      //   dest: pair.to_owned().into(),
      //   src,
      // }, line_dbg!("here"));

      Ok(())
    },
    TypeKind::Weak { .. } => todo!(),
    TypeKind::ReferenceTo { r#mut, .. } => {
      let pair = pair.dereference(store, *r#mut)?
        .expect("to dereference the type");

      default_types_of_type_pair(store, &pair, tasks)
    },
    | TypeKind::SizedArrayOf { ty, .. }
    | TypeKind::UnsizedArrayOf { ty, .. } => {
      ty::default_types_of_type(store, &TypeReference::Part(*ty), tasks)
    },
    TypeKind::Struct { prototype } => {
      let borrow = prototype.rget_from(store);

      for id in 0..borrow.members.len() {
        default_types_of_type(store, &TypeReference::StructMember(*prototype, id), tasks)?;
      };

      Ok(())
    },
  }
}
