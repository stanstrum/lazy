pub mod unknown;

use ::lang::intrinsic::Intrinsic;
use crate::lang::ty::{QualifiedSearchSpace};
use crate::resolve::TypePair;
use crate::tokenize::token::StringKind;

use super::*;

// impl DereferenceType for TypePair {
//   fn dereference(&self, lazy: &Lazy, r#mut: bool) -> Result<Option<Type>> {
//     let modifiers = self.modifiers.clone();
//     let reference = *self.pair.0;

//     clone.modifiers.push(TypePairModifier::Dereference);

//     Ok(Some(clone))
//   }
// }

impl TypeOf for TypePair {
  fn type_of(&self, lazy: &Lazy) -> Option<Type> {
    self.ty.type_of(lazy)
  }

  fn reference(&self, _lazy: &Lazy) -> Option<OverwriteTypeReference> {
    Some(self.clone().into())
  }
}

impl Resolve for TypePair {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let description = format!(line_dbg!("Resolve TypePair:\n- Ref.: {}\n- Type: {}"),
      self.overwrite.print(lazy),
      self.ty.print(lazy),
    );

    tasks.work(description, |tasks| {
      match &self.ty {
        Type::Unresolved { module, qualified } => {
          if let Some(ty) = unknown::resolve_qualified_to_type(lazy, *module, qualified, tasks)? {
            tasks.push(tasks::Subjugate {
              prerequisite: Box::new(tasks::OverwriteType {
                dest: self.clone().into(),
                src: ty,
              }),
              after: Box::new(tasks::ResolveAsTask::<TypeReference> {
                reference: self.overwrite.reference,
              }),
            }, line_dbg!("here"));
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
          => ty.resolve(lazy, tasks),
        | Type::Reference(reference) => {
          let ty = reference.rget_from(lazy).clone();

          let mut new_reference = self.clone();
          new_reference.ty = ty;

          new_reference.resolve(lazy, tasks)
        },
        Type::Struct { prototype } => prototype.resolve(lazy, tasks),
      }
    })
  }
}

impl Coerce for TypePair {
  fn coerce(&self, lazy: &Lazy, other_ref: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let a = self.overwrite.print(lazy);
    let b = self.ty.print(lazy);
    let c = other_ref.type_of(lazy)
      .map(|x| x.print(lazy))
      .unwrap_or_else(|| "{none}".into());
    let d = format!("{:?}", &self.overwrite.modifiers);

    let description = format!(
      line_dbg!("Coerce TypePair\n- Reference: {}\n- Modifiers: {}\n- Type:      {}\n- Coerce w/: {}"),
      a, d, b, c,
    );

    tasks.work(description, |tasks| {
      // println!(line_dbg!("here:\n{}"), tasks.explain(2));

      let Some(other) = other_ref.type_of(lazy) else {
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
          tasks.push(tasks::OverwriteType {
            dest: self.clone().into(),
            src: other,
          }, line_dbg!("here"));

          Ok(())
        },
        (Type::Intrinsic { kind, .. }, Type::WeakInteger { .. }) if kind.is_integer() => {
          Ok(())
        },
        (Type::Resolved { part, .. }, _) => {
          let ty = part.rget_from(lazy);
          let reference = TypeReference::Part(*part);

          TypePair::new(reference, ty.clone()).coerce(lazy, other_ref, tasks)
        },
        (Type::Reference(reference), _) => {
          let ty = reference.rget_from(lazy);
          TypePair::new(*reference, ty.clone()).coerce(lazy, other_ref, tasks)
        },
        (_, Type::Resolved { part, .. }) => {
          let ty = part.rget_from(lazy);
          let reference = TypeReference::Part(*part);
          let other_ref = TypePair::new(reference, ty.clone());
          self.coerce(lazy, &other_ref, tasks)
        },
        (_, Type::Reference(reference)) => {
          let ty = reference.rget_from(lazy);
          let other_ref = TypePair::new(*reference, ty.clone());
          self.coerce(lazy, &other_ref, tasks)
        },
        // SPONGE: structs can be equivalent to one another without being the
        //         exact same ...
        (Type::Struct { prototype: lhs }, Type::Struct { prototype: rhs }) if lhs == rhs => {
          Ok(())
        },
        (Type::Weak { .. }, _) => {
          tasks.push(tasks::OverwriteType {
            dest: self.overwrite.clone(),
            src: other,
          }, line_dbg!("here"));

          Ok(())
        },
        (_, Type::Unresolved { .. })
          => {
          // An unresolved doesn't tell us much
          Ok(())
        },
        (Type::Unresolved { module, qualified }, _) if qualified.is_implicit() => {
          let dest = self.clone().into();

          let mut qualified = qualified.clone();
          qualified.implicit = QualifiedSearchSpace::Type(other_ref.reference(lazy).expect("god help me"));

          let src = Type::Unresolved {
            module: *module,
            qualified,
          };

          // RHS should be anything but another Unresolved.  Try to resolve
          // an implicit
          let task = tasks::Subjugate {
            prerequisite: Box::new(tasks::OverwriteType {
              dest,
              src,
            }),
            after: Box::new(tasks::ResolveAsTask {
              reference: self.overwrite.reference,
            }),
          };

          tasks.push(task, line_dbg!("here"));

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
          ty_a.coerce(lazy, ty_b, tasks)?;
          ty_b.coerce(lazy, ty_a, tasks)?;

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

          ty.coerce(lazy, &Type::Intrinsic {
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
          ty.coerce(lazy, &Type::Intrinsic {
            kind: (*kind).into(),
            span: *span,
          }, tasks)
        },
        (a, b) => {
          // #[cfg(debug_assertions)] dbg!(a, b);

          if
            let Some(a) = self.dereference(lazy, false)? &&
            let Some(b) = other_ref.dereference(lazy, false)?
          {
            return a.coerce(lazy, &b, tasks);
          };

          tasks.seed_error(ErrorBase::TypeMismatch {
            whence: line_dbg!(),
            a_print: a.print(lazy),
            a_span: a.get_span(lazy),
            b_print: b.print(lazy),
            b_span: b.get_span(lazy),
          })
        },
      }
    })
  }
}

pub(in crate::resolve::impls) fn default_types_of_type_pair(lazy: &mut Lazy, pair: &TypePair, tasks: &mut Tasks) -> Result<()> {
  match &pair.ty {
    Type::Reference(ty) => default_types_of_type(lazy, ty, tasks),
    Type::Resolved { part, .. } => default_types_of_type(
      lazy, &TypeReference::Part(*part), tasks,
    ),
    Type::Unresolved { .. } => todo!(),
    Type::Intrinsic { .. } => Ok(()),
    Type::WeakInteger { span, .. } => {
      let src = Type::Intrinsic {
        kind: Intrinsic::U32,
        span: *span,
      };

      tasks.push(tasks::OverwriteType {
        dest: pair.to_owned().into(),
        src,
      }, line_dbg!("here"));

      Ok(())
    },
    Type::WeakFloat { .. } => todo!(),
    &Type::WeakString { kind, characters, dereferenced, span } => {
      // dbg!(kind, characters, dereferenced);

      if dereferenced {
        todo!()
      };

      let src = {
        let parent_module = pair.overwrite.reference.parent_module(lazy);

        let element_intrinsic = kind.into();
        let element_part = Type::Intrinsic { kind: element_intrinsic, span };
        let element_reference = parent_module.add_type_part(element_part, lazy);

        let arr_of_element_part = Type::SizedArrayOf { ty: element_reference, size: characters, span };
        let arr_of_element_reference = parent_module.add_type_part(arr_of_element_part, lazy);

        Type::ReferenceTo { ty: arr_of_element_reference, r#mut: false, span }
      };

      tasks.push(tasks::OverwriteType {
        dest: pair.to_owned().into(),
        src,
      }, line_dbg!("here"));

      Ok(())
    },
    Type::Weak { .. } => todo!(),
    Type::ReferenceTo { r#mut, .. } => {
      let pair = pair.dereference(lazy, *r#mut)?
        .expect("to dereference the type");

      default_types_of_type_pair(lazy, &pair, tasks)
    },
    | Type::SizedArrayOf { ty, .. }
    | Type::UnsizedArrayOf { ty, .. } => {
      ty::default_types_of_type(lazy, &TypeReference::Part(*ty), tasks)
    },
    Type::Struct { prototype } => {
      let borrow = prototype.rget_from(lazy);

      for id in 0..borrow.members.len() {
        default_types_of_type(lazy, &TypeReference::StructMember(*prototype, id), tasks)?;
      };

      Ok(())
    },
  }
}
