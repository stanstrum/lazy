use crate::lang::reference::{ExpressionReference, TypeReference, VariableReference};
use crate::resolve::SpecialPair;

use super::*;

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

    tasks.work(description, |tasks| {
      TypeReference::Expression(*self).coerce(lazy, other, tasks)

      // match self.rget_from(lazy) {
      //   Expression::Block(_) => todo!(),
      //   Expression::Literal { .. } => todo!(),
      //   Expression::Variable { reference, .. } => reference.coerce(lazy, other, tasks),
      //   Expression::Unknown { .. } => todo!(),
      //   Expression::Unary { .. } => todo!(),
      //   Expression::Binary { .. } => todo!(),
      // }
    })
  }
}

