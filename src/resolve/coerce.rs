use crate::resolve::type_of::TypeOf;
use crate::aster::pprint::Pretty;

use super::*;

pub trait Coerce {
  fn coerce(&self, lazy: &Lazy, other: &(impl TypeOf + Coerce), tasks: &mut Tasks) -> Result<()>;
}

pub struct SpecialPair<'a, S: Store<R>, R: Reference<S>>(
  pub &'a R,
  pub &'a S::Out,
);

pub type TypePair<'a, 'b> = SpecialPair<'a, Lazy<'b>, TypeReference>;

impl<'a, S: Store<R>, R: Reference<S>> TypeOf for SpecialPair<'a, S, R> where S::Out: TypeOf {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    self.1.type_of(lazy)
  }
}

impl<'a, 'b> Coerce for TypePair<'a, 'b> {
  fn coerce(&self, lazy: &Lazy, other_ref: &(impl TypeOf + Coerce), tasks: &mut Tasks) -> Result<()> {
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
        tasks.push(ResolveType {
          dest: **reference,
          value: other,
        });

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
        let other_ref = SpecialPair(&reference, ty);
        self.coerce(lazy, &other_ref, tasks)
      },
      (_, Type::Reference(reference)) => {
        let ty = reference.rget_from(lazy);
        let other_ref = SpecialPair(reference, ty);
        self.coerce(lazy, &other_ref, tasks)
      },
      (Type::Weak { .. }, _) => {
        tasks.push(ResolveType {
          dest: **reference,
          value: other,
        });

        Ok(())
      },
      (_, Type::Unresolved { .. }) => Ok(()),
      (a, b) => {
        println!("{a:?}");
        println!("{b:?}");

        let a = a.print(lazy);
        let b = b.print(lazy);

        panic!("cannot coerce {a} with {b}")
      },
    }
  }
}
