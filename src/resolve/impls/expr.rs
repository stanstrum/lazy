use crate::lang::expr::{BlockExpression, Expression, Variable};
use crate::lang::expr::operator::BinaryOperator;
use crate::lang::reference::{BlockReference, ExpressionReference, TypeReference, VariableReference};
use crate::lang::ty::{Intrinsic, Type};
use crate::resolve::SpecialPair;

use super::*;

impl TypeOf for Variable {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    self.ty.type_of(lazy)
  }
}

impl TypeOf for Expression {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    match self {
      Expression::Block(block) => block.type_of(lazy),
      Expression::Variable { reference, .. } => reference.type_of(lazy),
      // TODO: again, very unsure about this... we are relying on the Resolve
      //       mechanism to hit the insides of the Expression and then
      //       looping to finish the job.  is this Functional™?
      | Expression::Literal { out, .. }
      | Expression::Unknown { out, .. }
      | Expression::Unary { out, .. }
      | Expression::Binary { out, .. }
        => out.type_of(lazy)
    }
  }
}

impl TypeOf for BlockExpression {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    // TODO: is this correct? should I try to match the expr type directly,
    //       maybe in addition to this?  Coerce in TypeOf? what could go
    //       wrong ???
    self.out.type_of(lazy)
  }
}

impl Resolve for BlockReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let description = format!(line_dbg!("Resolve BlockReference: {}"), self.print(lazy));

    tasks.work(description, |tasks| {
      let borrow = self.rget_from(lazy);

      for id in 0..borrow.variables.len() {
        VariableReference::Block(*self, id).resolve(lazy, tasks)?;
      };

      for &expr in borrow.children.iter() {
        ExpressionReference(*self, expr).resolve(lazy, tasks)?;
      };

      Ok(())
    })
  }
}

impl Resolve for ExpressionReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let description = {
      format!(line_dbg!("Resolve ExpressionReference: {} in {}"),
        self.print(lazy),
        self.0.print(lazy),
      )
    };

    tasks.work(description, |tasks| {
      let borrow = self.rget_from(lazy);
        let ty_reference = TypeReference::Expression(*self);

      match borrow {
        Expression::Block(block) => {
          block.resolve(lazy, tasks)
        },
        Expression::Literal { out, .. } => {
          SpecialPair(&ty_reference, out).resolve(lazy, tasks)
        },
        Expression::Variable { reference, .. } => {
          reference.resolve(lazy, tasks)
        },
        Expression::Binary {
          a, b,
          op: (BinaryOperator::Assign, op_span),
          out,
          ..
        } => {
          let out_pair = SpecialPair(&ty_reference, out);

          let void_op = Type::Intrinsic {
            kind: Intrinsic::Void,
            span: *op_span,
          };

          out_pair.coerce(lazy, &void_op, tasks)?;
          out_pair.resolve(lazy, tasks)?;

          a.coerce(lazy, b, tasks)?;
          b.coerce(lazy, a, tasks)?;

          a.resolve(lazy, tasks)?;
          b.resolve(lazy, tasks)?;

          Ok(())
        },
        Expression::Unknown { qualified, .. } if qualified.parts.len() == 1 && !qualified.implicit => {
          let part = qualified.parts.first().unwrap();

          let mut block = Some(self.0);

          let hierarchy = std::iter::from_fn(move || {
            let old = block;
            block = block.and_then(|block| lazy.rget(block).parent);

            old
          });

          let variables_iter_iter = hierarchy.map(|block| {
            lazy.rget(block)
              .variables.iter().enumerate()
              // I do not understand why this lambda is `move` ...
              .map(move |(id, var)| (var.name.id, VariableReference::Block(block, id))
            )
          });

          let function = self.0.0;
          let arguments_iter = lazy.rget(function)
            .header.arguments.iter()
            .enumerate().map(|(id, arg)| {
              (arg.name.id, VariableReference::Argument(function, id))
            }
          );

          let name_reference_iter = variables_iter_iter.flatten().chain(arguments_iter);

          // name_reference_iter.inspect(|p| { dbg!(p); });

          for (pool_id, variable_reference) in name_reference_iter {
            if part.id == pool_id {
              tasks.push(tasks::OverwriteExpression {
                dest: *self,
                src: Expression::Variable {
                  reference: variable_reference,
                  span: borrow.get_span(lazy),
                },
              }, line_dbg!("here"));

              return Ok(());
            };
          };

          Err(Box::new(Error::UnknownTypeName {
            module_name: lazy.describe_module(self.0.0.rget_from(lazy).parent),
            span: borrow.get_span(lazy),
          }))
        },
        _ => todo!("{borrow:?}\n{}", borrow.print_with(lazy.rget(self.0.0), lazy).collect::<Vec<_>>().join("\n")),
      }
    })
  }
}

