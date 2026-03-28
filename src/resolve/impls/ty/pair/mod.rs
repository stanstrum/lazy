mod unknown;

use crate::lang::ty::{Intrinsic, QualifiedSearchSpace};
use crate::resolve::TypePair;

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
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    self.ty.type_of(lazy)
  }

  fn reference(&self, lazy: &Lazy) -> Option<OverwriteTypeReference> {
    Some(self.clone().into())
  }
}

impl Resolve for TypePair {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let description = format!(line_dbg!("Resolve TypePair:\n- Ref.: {}\n- Type: {}"),
      self.reference.print(lazy),
      self.ty.print(lazy),
    );

    tasks.work(description, |tasks| {
      match &self.ty {
        Type::Unresolved { module, qualified } => {
          if let Some(ty) = unknown::resolve_qualified_to_type(lazy, *module, qualified)? {
            tasks.push(tasks::Subjugate {
              prerequisite: Box::new(tasks::OverwriteType {
                dest: self.clone().into(),
                src: ty,
              }),
              after: Box::new(tasks::ResolveAsTask::<TypeReference> {
                reference: self.reference,
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
      }
    })
  }
}

impl TypeOf for OverwriteTypeReference {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    let mut ty = self.reference.rget_from(lazy).to_owned();

    for modifier in self.modifiers.iter() {
      match modifier {
        TypePairModifier::Dereference => match ty {
          Type::Reference(_) => todo!(),
          Type::Resolved { .. } => todo!(),
          Type::Unresolved { .. } => todo!(),
          Type::Intrinsic { .. } => todo!(),
          Type::WeakInteger { .. } => todo!(),
          Type::WeakFloat { .. } => todo!(),
          Type::WeakString { .. } => todo!(),
          Type::Weak { .. } => todo!(),
          Type::ReferenceTo { .. } => todo!(),
          Type::UnsizedArrayOf { .. } => todo!(),
          Type::SizedArrayOf { .. } => todo!(),
        },
      }
    };

    Ok(Some(ty))
  }

  fn reference(&self, lazy: &Lazy) -> Option<OverwriteTypeReference> {
    todo!()
  }
}

impl Coerce for OverwriteTypeReference {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let Some(ty) = self.type_of(lazy)? else {
      return Ok(());
    };

    TypePair {
      reference: self.reference,
      modifiers: self.modifiers.clone(),
      ty,
    }.coerce(lazy, other, tasks)
  }
}

impl Coerce for TypePair {
  fn coerce(&self, lazy: &Lazy, other_ref: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let a = self.reference.print(lazy);
    let b = self.ty.print(lazy);
    let c = other_ref.type_of(lazy)?.map(|x| x.print(lazy)).unwrap_or_else(|| "{none}".into());
    let d = format!("{:?}", &self.modifiers);

    let description = format!(
      line_dbg!("Coerce TypePair\n- Reference: {}\n- Modifiers: {}\n- Type:      {}\n- Coerce w/: {}"),
      a, d, b, c,
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
              dest: OverwriteTypeReference {
                reference: self.reference,
                modifiers: self.modifiers.clone(),
              },
              src: other,
            }, line_dbg!("here"));

            Ok(())
          },
          (Type::Intrinsic { kind, .. }, Type::WeakInteger { .. }) if kind.is_integer() => {
            Ok(())
          },
          (Type::Resolved { part, .. }, _) => {
            let ty = part.rget_from(lazy);
            TypePair::new(self.reference, ty.clone()).coerce(lazy, other_ref, tasks)
          },
          (Type::Reference(reference), _) => {
            let ty = reference.rget_from(lazy);
            TypePair::new(self.reference, ty.clone()).coerce(lazy, other_ref, tasks)
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
          (Type::Weak { .. }, _) => {
            tasks.push(tasks::OverwriteType {
              dest: OverwriteTypeReference {
                reference: self.reference,
                modifiers: self.modifiers.clone(),
              },
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
                reference: self.reference,
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
              kind: kind.into_intrinsic(),
              span: *span,
            }, tasks)?;

            Ok(())
          },
          (a, b) => {
            dbg!(a, b);

            if
              let Some(a) = self.dereference(lazy, false)? &&
              let Some(b) = other_ref.dereference(lazy, false)?
            {
              return a.coerce(lazy, &b, tasks);
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
