use crate::lang::reference::{ExpressionReference, TypeReference, VariableReference};
use crate::resolve::SpecialPair;

use super::*;

impl Resolve for VariableReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let description = {
      let (function, print): (_, &dyn Pretty<Out = String>) = match self {
        VariableReference::Block(block_reference, _) => (block_reference.0, block_reference),
        VariableReference::Argument(function_reference, _) => (*function_reference, function_reference),
      };

      format!(
        line_dbg!("Resolve VariableReference: {} in {}"),
        function.print(lazy),
        print.print(lazy),
      )
    };

    tasks.work(description, |tasks| {
      TypeReference::Variable(*self).resolve(lazy, tasks)
    })
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

