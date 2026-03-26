mod unknown;

use crate::lang::ty::{Intrinsic, QualifiedSearchSpace};
use crate::resolve::{SpecialPair, TypePair, TypePairModifier};

use super::*;

impl<'a, S: Store<R>, R: Reference<S>> TypeOf for SpecialPair<'a, S, R> where S::Out: TypeOf {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    self.1.type_of(lazy)
  }

  fn reference(&self, lazy: &Lazy) -> Option<TypeReference> {
    self.1.reference(lazy)
  }
}

// impl<'a, 'b> DereferenceType for TypePair<'a, 'b> {
//   fn dereference(&self, lazy: &Lazy, r#mut: bool) -> Result<Option<Type>> {
//     let modifiers = self.modifiers.clone();
//     let reference = *self.pair.0;


//     clone.modifiers.push(TypePairModifier::Dereference);

//     Ok(Some(clone))
//   }
// }

impl<'a, 'b> Resolve for TypePair<'a, 'b> {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let SpecialPair(reference, ty) = &self.pair;

    let description = format!(line_dbg!("Resolve TypePair:\n- Ref.: {}\n- Type: {}"),
      reference.print(lazy),
      ty.print(lazy),
    );

    tasks.work(description, |tasks| {
      match ty {
        Type::Unresolved { module, qualified } => {
          if let Some(ty) = unknown::resolve_qualified_to_type(lazy, *module, qualified)? {
            tasks.push(tasks::Subjugate {
              prerequisite: Box::new(tasks::OverwriteType {
                dest: (**reference).into(),
                src: ty,
              }),
              after: Box::new(tasks::ResolveAsTask::<TypeReference> {
                reference: **reference,
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
          let ty = reference.rget_from(lazy);
          TypePair::new(reference, ty).resolve(lazy, tasks)
        },
      }
    })
  }
}

impl<'a, 'b> Coerce for TypePair<'a, 'b> {
  fn coerce(&self, lazy: &Lazy, other_ref: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let a = self.pair.0.print(lazy);
    let b = self.pair.1.print(lazy);
    let c = other_ref.type_of(lazy)?.map(|x| x.print(lazy)).unwrap_or_else(|| "{none}".into());

    let description = format!(
      line_dbg!("Coerce TypePair\n- Reference: {}\n- Type:      {}\n- Coerce w/: {}"),
      a, b, c
    );

    tasks.work(description, |tasks| {
        // println!(line_dbg!("here:\n{}"), tasks.explain(2));

        let Some(other) = other_ref.type_of(lazy)? else {
          return Ok(());
        };

        // println!(
        //   line_dbg!("TypePair({}, {}) coerced by {}"),
        //   self.0.print(lazy),
        //   self.1.print(lazy),
        //   other.print(lazy),
        // );

        let SpecialPair(reference, ty) = self.pair;

        match (ty, &other) {
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
              dest: (*reference).into(),
              src: other,
            }, line_dbg!("here"));

            Ok(())
          },
          (Type::Intrinsic { kind, .. }, Type::WeakInteger { .. }) if kind.is_integer() => {
            Ok(())
          },
          (Type::Resolved { part, .. }, _) => {
            let ty = part.rget_from(lazy);
            TypePair::new(reference, ty).coerce(lazy, other_ref, tasks)
          },
          (Type::Reference(reference), _) => {
            let ty = reference.rget_from(lazy);
            TypePair::new(reference, ty).coerce(lazy, other_ref, tasks)
          },
          (_, Type::Resolved { part, .. }) => {
            let ty = part.rget_from(lazy);
            let reference = TypeReference::Part(*part);
            let other_ref = TypePair::new(&reference, ty);
            self.coerce(lazy, &other_ref, tasks)
          },
          (_, Type::Reference(reference)) => {
            let ty = reference.rget_from(lazy);
            let other_ref = TypePair::new(reference, ty);
            self.coerce(lazy, &other_ref, tasks)
          },
          (Type::Weak { .. }, _) => {
            tasks.push(tasks::OverwriteType {
              dest: (*reference).into(),
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
                reference: *reference,
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
          (a, b) => {
            dbg!(a, b);

            if
              let Some(a) = dbg!(a.dereference(lazy, false)?) &&
              let Some(b) = dbg!(b.dereference(lazy, false)?)
            {
              return todo!("coerce -> dereference");
            };

            Err(Box::new(Error::TypeMismatch {
              whence: line_dbg!(""),
              a_print: a.print(lazy),
              a_span: a.get_span(lazy),
              b_print: b.print(lazy),
              b_span: b.get_span(lazy),
            }))
          },
        }
      }
    )
  }
}
