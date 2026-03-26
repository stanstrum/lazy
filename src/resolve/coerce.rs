use crate::lang::expr::Expression;
use crate::lang::reference::{ExpressionReference, Reference, Store, TypeReference, VariableReference};
use crate::lang::span::GetSpan;
use crate::resolve::type_of::TypeOf;
use crate::aster::pprint::Pretty;

use super::*;

pub trait Coerce {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()>;
}

#[derive(Debug)]
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
  fn coerce(&self, lazy: &Lazy, other_ref: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let a = self.0.print(lazy);
    let b = self.1.print(lazy);
    let c = other_ref.type_of(lazy)?.map(|x| x.print(lazy)).unwrap_or_else(|| "{none}".into());

    task_work(tasks, format!(line_dbg!("Coerce TypePair\n- Reference: {}\n- Type:      {}\n- Coerce w/: {}"), a, b, c),
      |tasks| {

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
            tasks.push(OverwriteType {
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
            tasks.push(OverwriteType {
              dest: **reference,
              src: other,
            }, line_dbg!("here"));

            Ok(())
          },
          (_, Type::Unresolved { .. }) => Ok(()),
          (a, b) => {
            dbg!(a, b);

            Err(Box::new(Error::TypeMismatch {
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

impl Coerce for VariableReference {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let ty = &self.rget_from(lazy).ty;
    let reference = TypeReference::Variable(*self);

    SpecialPair(&reference, ty).coerce(lazy, other, tasks)
  }
}

impl Coerce for ExpressionReference {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let a = self.print(lazy);
    let b = other.type_of(lazy)?.map(|x| x.print(lazy)).unwrap_or_else(|| "{none}".into());

    let description = format!(line_dbg!("Coerce ExpressionReference\n- Reference: {}\n- Coerce w/: {}"), a, b);

    task_work(tasks, description, |tasks| {
      match self.rget_from(lazy) {
        Expression::Block(_) => todo!(),
        Expression::Literal { .. } => todo!(),
        Expression::Variable { reference, .. } => reference.coerce(lazy, other, tasks),
        Expression::Unknown { .. } => todo!(),
        Expression::Unary { .. } => todo!(),
        Expression::Binary { .. } => todo!(),
      }
    })
  }
}
