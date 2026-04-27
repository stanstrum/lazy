pub mod unknown;

use lang::Compiler;
use lang::token::StringKind;
use lang::module::AddTypePart;
use lang::intrinsic::Intrinsic;
use lang::ty::{QualifiedSearchSpace, TypePair};

use super::*;

// impl DereferenceType for TypePair {
//   fn dereference(&self, lazy: &C::Store<'_>, r#mut: bool) -> Result<C, Option<Type>> {
//     let modifiers = self.modifiers.clone();
//     let reference = *self.pair.0;

//     clone.modifiers.push(TypePairModifier::Dereference);

//     Ok(Some(clone))
//   }
// }

impl<C: Compiler + 'static> Resolve<C> for TypePair<C> {
  fn resolve(&self, store: &C::Store<'_>, tasks: &mut Tasks<C>) -> Result<C> {
    let description = format!(line_dbg!("Resolve TypePair:\n- Ref.: {}\n- Type: {}"),
      self.overwrite.print(store),
      self.ty.print(store),
    );

    tasks.work(description, |tasks| {
      match &self.ty {
        Type::Unresolved { module, qualified } => {
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
        Type::Intrinsic { .. } => {
          // do nothing ...
          Ok(())
        },
        | Type::WeakInteger { .. }
        | Type::WeakFloat { .. }
        | Type::WeakString { .. }
        | Type::Weak { .. } => {
          // do nothing ... can't resolve this
          Ok(())
        }
        | Type::ReferenceTo { ty, .. }
        | Type::UnsizedArrayOf { ty, .. }
        | Type::SizedArrayOf { ty, .. }
        | Type::Resolved { part: ty, .. }
          => ty.resolve(store, tasks),
        | Type::Reference(reference) => {
          let ty = reference.rget_from(store).clone();

          let mut new_reference = self.clone();
          new_reference.ty = ty;

          new_reference.resolve(store, tasks)
        },
        Type::Struct { prototype } => prototype.resolve(store, tasks),
      }
    })
  }
}

impl<C: Compiler + 'static> Coerce<C> for TypePair<C> {
  fn coerce(&self, store: &C::Store<'_>, other_ref: &impl TypeOf<C>, tasks: &mut Tasks<C>) -> Result<C> {
    let a = self.overwrite.print(store);
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
        (Type::Intrinsic { kind: kind_a, .. }, Type::Intrinsic { kind: kind_b, .. })
          if kind_a == kind_b
        => {
          // do nothing
          Ok(())
        },
        (
          | Type::WeakInteger { .. }
          | Type::WeakFloat { .. },
          Type::Intrinsic { kind, .. },
        ) if !matches!(kind, Intrinsic::Bool | Intrinsic::Void) => {
          todo!();
          // tasks.push(tasks::OverwriteType {
          //   dest: self.clone().into(),
          //   src: other,
          // }, line_dbg!("here"));

          Ok(())
        },
        (Type::Intrinsic { kind, .. }, Type::WeakInteger { .. }) if kind.is_integer() => {
          Ok(())
        },
        (Type::Resolved { part, .. }, _) => {
          let ty = part.rget_from(store);
          let reference = TypeReference::Part(*part);

          TypePair::new(reference, ty.clone()).coerce(store, other_ref, tasks)
        },
        (Type::Reference(reference), _) => {
          let ty = reference.rget_from(store);
          TypePair::new(*reference, ty.clone()).coerce(store, other_ref, tasks)
        },
        (_, Type::Resolved { part, .. }) => {
          let ty = part.rget_from(store);
          let reference = TypeReference::Part(*part);
          let other_ref = TypePair::new(reference, ty.clone());
          self.coerce(store, &other_ref, tasks)
        },
        (_, Type::Reference(reference)) => {
          let ty = reference.rget_from(store);
          let other_ref = TypePair::new(*reference, ty.clone());
          self.coerce(store, &other_ref, tasks)
        },
        // SPONGE: structs can be equivalent to one another without being the
        //         exact same ...
        (Type::Struct { prototype: lhs }, Type::Struct { prototype: rhs }) if lhs == rhs => {
          Ok(())
        },
        (Type::Weak { .. }, _) => {
          todo!();

          // tasks.push(tasks::OverwriteType {
          //   dest: self.overwrite.clone(),
          //   src: other,
          // }, line_dbg!("here"));

          Ok(())
        },
        (_, Type::Unresolved { .. })
          => {
          // An unresolved doesn't tell us much
          Ok(())
        },
        (Type::Unresolved { module, qualified }, _) if qualified.is_implicit() => {
          let dest = /* self.clone().into() */ todo!();

          let mut qualified = qualified.clone();
          todo!();
          // qualified.implicit = QualifiedSearchSpace::Type(other_ref.reference(store).expect("god help me"));

          let src = Type::Unresolved {
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
        (Type::Unresolved { .. }, _) => {
          // Can't just throw an error here.  A task could have yet to come
          // around and update this.  Let the resolver or verifier sort out
          // this mess.
          Ok(())
        },
        (
          Type::SizedArrayOf { ty: ty_a, .. } | Type::UnsizedArrayOf { ty: ty_a, .. },
          Type::SizedArrayOf { ty: ty_b, .. } | Type::UnsizedArrayOf { ty: ty_b, .. },
        ) => {
          ty_a.coerce(store, ty_b, tasks)?;
          ty_b.coerce(store, ty_a, tasks)?;

          Ok(())
        },
        (
          Type::WeakString { kind: kind_a, characters: characters_a, dereferenced: dereferenced_a, .. },
          Type::WeakString { kind: kind_b, characters: characters_b, dereferenced: dereferenced_b, .. },
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
          Type::SizedArrayOf { ty, size, .. },
          Type::WeakString { kind, characters, dereferenced, span },
        )
        | (
          Type::WeakString { kind, characters, dereferenced, span },
          Type::SizedArrayOf { ty, size, .. },
        ) /* if size == characters */ => {
          assert!(dereferenced, "todo");
          assert!(size == characters, "throw error for weak string size mismatch");

          ty.coerce(store, &Type::Intrinsic {
            kind: (*kind).into(),
            span: *span,
          }, tasks)?;

          Ok(())
        },
        | (
          Type::UnsizedArrayOf { ty, .. },
          Type::WeakString { kind, span, .. },
        )
        | (
          Type::WeakString { kind, span, .. },
          Type::UnsizedArrayOf { ty, .. },
        ) => {
          ty.coerce(store, &Type::Intrinsic {
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

pub(in crate::impls) fn default_types_of_type_pair<C: Compiler + 'static>(store: &mut C::Store<'_>, pair: &TypePair<C>, tasks: &mut Tasks<C>) -> Result<C> {
  match &pair.ty {
    Type::Reference(ty) => default_types_of_type(store, ty, tasks),
    Type::Resolved { part, .. } => default_types_of_type(
      store, &TypeReference::Part(*part), tasks,
    ),
    Type::Unresolved { .. } => todo!(),
    Type::Intrinsic { .. } => Ok(()),
    Type::WeakInteger { span, .. } => {
      let src = Type::Intrinsic {
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
    Type::WeakFloat { .. } => todo!(),
    &Type::WeakString { kind, characters, dereferenced, span } => {
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
    Type::Weak { .. } => todo!(),
    Type::ReferenceTo { r#mut, .. } => {
      let pair = pair.dereference(store, *r#mut)?
        .expect("to dereference the type");

      default_types_of_type_pair(store, &pair, tasks)
    },
    | Type::SizedArrayOf { ty, .. }
    | Type::UnsizedArrayOf { ty, .. } => {
      ty::default_types_of_type(store, &TypeReference::Part(*ty), tasks)
    },
    Type::Struct { prototype } => {
      let borrow = prototype.rget_from(store);

      for id in 0..borrow.members.len() {
        default_types_of_type(store, &TypeReference::StructMember(*prototype, id), tasks)?;
      };

      Ok(())
    },
  }
}
