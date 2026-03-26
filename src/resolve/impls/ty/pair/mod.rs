mod unknown;
use crate::{lang::ty::Intrinsic, resolve::TypePair};

use super::*;

impl<'a, S: Store<R>, R: Reference<S>> TypeOf for SpecialPair<'a, S, R> where S::Out: TypeOf {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    self.1.type_of(lazy)
  }
}

impl<'a, 'b> Resolve for TypePair<'a, 'b> {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let SpecialPair(reference, ty) = self;

    let description = format!(line_dbg!("Resolve TypePair:\n- Ref.: {}\n- Type: {}"),
      reference.print(lazy),
      ty.print(lazy),
    );

    tasks.work(description, |tasks| {
      match ty {
        Type::Unresolved { module, qualified } => {
          if let Some(ty) = unknown::resolve_qualified_to_type(lazy, *module, qualified)? {
            tasks.push(tasks::Subjugate {
              after: Box::new(tasks::ResolveAsTask::<TypeReference> {
                reference: **reference,
              }),
              prerequisite: Box::new(tasks::OverwriteType {
                dest: **reference,
                src: ty,
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
          SpecialPair(reference, ty).resolve(lazy, tasks)
        },
      }
    })
  }
}

impl<'a, 'b> Coerce for TypePair<'a, 'b> {
  fn coerce(&self, lazy: &Lazy, other_ref: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let a = self.0.print(lazy);
    let b = self.1.print(lazy);
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

        let Self(reference, ty) = self;

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
              dest: **reference,
              src: other,
            }, line_dbg!("here"));

            Ok(())
          },
          (Type::Resolved { part, .. }, _) => {
            let ty = part.rget_from(lazy);
            SpecialPair::<Lazy, TypeReference>(reference, ty).coerce(lazy, other_ref, tasks)
          },
          (Type::Reference(reference), _) => {
            let ty = reference.rget_from(lazy);
            SpecialPair::<Lazy, TypeReference>(reference, ty).coerce(lazy, other_ref, tasks)
          },
          (_, Type::Resolved { part, .. }) => {
            let ty = part.rget_from(lazy);
            let reference = TypeReference::Part(*part);
            let other_ref: TypePair = SpecialPair(&reference, ty);
            self.coerce(lazy, &other_ref, tasks)
          },
          (_, Type::Reference(reference)) => {
            let ty = reference.rget_from(lazy);
            let other_ref: TypePair = SpecialPair(reference, ty);
            self.coerce(lazy, &other_ref, tasks)
          },
          (Type::Weak { .. }, _) => {
            tasks.push(tasks::OverwriteType {
              dest: **reference,
              src: other,
            }, line_dbg!("here"));

            Ok(())
          },
          (
            Type::ReferenceTo { ty: a, r#mut: false, .. },
            &Type::WeakString { kind, dereferenced: false, span, .. }
          ) => {
            let b = Type::Intrinsic {
              kind: kind.into_intrinsic(),
              span,
            };

            // SPONGE SPONGE SPONGE

            // weak, can't error
            Ok(())
          },
          (_, Type::Unresolved { .. }) => Ok(()),
          (a, b) => {
            dbg!(a, b);

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
