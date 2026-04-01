use crate::lang::expr::Expression;
use crate::lang::expr::operator::BinaryOperator;
use crate::lang::reference::{BlockReference, ExpressionReference, TypeReference, VariableReference};
use crate::lang::ty::{Intrinsic, Type};
use crate::resolve::TypePair;
use crate::resolve::tasks::OverwriteTypeReference;
use crate::tokenize::token::Span;

use super::*;

impl TypeOf for VariableReference {
  fn type_of(&self, lazy: &Lazy) -> Option<Type> {
    TypeReference::Variable(*self).type_of(lazy)
  }

  fn reference(&self, _lazy: &Lazy) -> Option<OverwriteTypeReference> {
    Some(TypeReference::Variable(*self).into())
  }
}

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
    TypeReference::Variable(*self).coerce(lazy, other, tasks)
  }
}

fn verify_variable(lazy: &Lazy, variable: VariableReference, tasks: &mut Tasks) -> Result<()> {
  let description = {
    let (function, print): (_, &dyn Pretty<Out = String>) = match &variable {
      VariableReference::Block(block_reference, _) => (block_reference.0, block_reference),
      VariableReference::Argument(function_reference, _) => (*function_reference, function_reference),
    };

    let function_borrow = lazy.rget(function);
    let parent = function_borrow.parent;

    format!(
      line_dbg!("Verify variable: {}::{} in {}"),
      lazy.describe_module(parent),
      function_borrow.header.name.print(lazy),
      print.print(lazy),
    )
  };

  tasks.work(description, |tasks| {
    ty::verify_typeof(lazy, &TypeReference::Variable(variable), tasks)
  })
}

impl TypeOf for ExpressionReference {
  fn type_of(&self, lazy: &Lazy) -> Option<Type> {
    match self.rget_from(lazy) {
      Expression::Block(block) => block.type_of(lazy),
      Expression::Variable { reference, .. } => reference.type_of(lazy),
      // TODO: again, very unsure about this... we are relying on the Resolve
      //       mechanism to hit the insides of the Expression and then
      //       looping to finish the job.  is this Functional™?
      | Expression::Literal { out, .. }
      | Expression::Unknown { out, .. }
      | Expression::Unary { out, .. }
      | Expression::Binary { out, .. }
        => {
          let reference = TypeReference::Expression(*self);

          TypePair::new(reference, out.clone()).type_of(lazy)
        },
    }
  }

  fn reference(&self, _lazy: &Lazy) -> Option<OverwriteTypeReference> {
    Some(TypeReference::Expression(*self).into())
  }
}

impl Coerce for ExpressionReference {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let a = self.print(lazy);
    let b = other.type_of(lazy).map(|x| x.print(lazy)).unwrap_or_else(|| "{none}".into());

    let description = format!(line_dbg!("Coerce ExpressionReference\n- Reference: {}\n- Coerce w/: {}"), a, b);

    tasks.work(description, |tasks| {
      // match self.rget_from(lazy) {
      //   Expression::Block(_) => todo!(),
      //   Expression::Literal { .. } => todo!(),
      //   Expression::Variable { reference, .. } => reference.coerce(lazy, other, tasks),
      //   Expression::Unknown { .. } => todo!(),
      //   Expression::Unary { .. } => todo!(),
      //   Expression::Binary { .. } => todo!(),
      // };

      TypeReference::Expression(*self).coerce(lazy, other, tasks)
    })
  }
}

impl TypeOf for BlockReference {
  fn type_of(&self, lazy: &Lazy) -> Option<Type> {
    // TODO: is this correct? should I try to match the expr type directly,
    //       maybe in addition to this?  Coerce in TypeOf? what could go
    //       wrong ???
    let reference = TypeReference::Block(*self);
    let ty = &self.rget_from(lazy).out;

    OverwriteTypeReference::from(TypePair::new(reference, ty.clone())).type_of(lazy)
  }

  fn reference(&self, _lazy: &Lazy) -> Option<OverwriteTypeReference> {
    Some(TypeReference::Block(*self).into())
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

pub(super) fn verify_block(lazy: &Lazy, block: &BlockReference, ret_ty: Option<&TypePair>, tasks: &mut Tasks) -> Result<()> {
  let block_borrow = block.rget_from(lazy);

  let description = {
    let Span { start, end, .. } = block_borrow.span;

    format!(line_dbg!("Verify block: {}:{} - {}:{}"),
      start.line, start.column,
      end.line, end.column,
    )
  };

  tasks.work(description, |tasks| {
    let block_type_reference = TypeReference::Block(*block);
    let block_out = TypePair::new(block_type_reference, block_borrow.out.clone());

    if let Some(ret_ty) = ret_ty {
      block_out.coerce(lazy, ret_ty, tasks)?;
    };

    let last = block_borrow.returns_last.then(|| *block_borrow.children.last().unwrap());
    let is_last = |id: &_| last.is_some_and(|x| x == *id);

    for id in block_borrow.children.iter() {
      let expr = ExpressionReference(*block, *id);

      let irr_reference = TypeReference::Expression(expr);
      let irr_ty = irr_reference.rget_from(lazy);

      let irr = TypePair::new(irr_reference, irr_ty.clone());

      let ret_ty = if is_last(id) && let Some(ret_ty) = ret_ty {
        irr.coerce(lazy, ret_ty, tasks)?;
        Some(ret_ty)
      } else {
        None
      };

      verify_expr(lazy, expr, ret_ty, tasks)?;
    };

    Ok(())
  })
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
          TypePair::new(ty_reference, out.clone()).resolve(lazy, tasks)
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
          let out_pair = TypePair::new(ty_reference, out.clone());

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
        Expression::Unknown { qualified, .. } if qualified.parts.len() == 1 && !qualified.is_implicit() => {
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

          tasks.seed_error(ErrorBase::UnknownTypeName {
            module_name: lazy.describe_module(self.0.0.rget_from(lazy).parent),
            span: borrow.get_span(lazy),
          })
        },
        _ => todo!("{borrow:?}\n{}", borrow.print_with(lazy.rget(self.0.0), lazy).collect::<Vec<_>>().join("\n")),
      }
    })
  }
}

fn verify_expr(lazy: &Lazy, expr: ExpressionReference, ret_ty: Option<&TypePair>, tasks: &mut Tasks) -> Result<()> {
  let Span { start, end , .. } = lazy.rget(expr).get_span(lazy);

  let description = format!(line_dbg!("Verify expr {}:{} - {}:{}"),
    start.line, start.column,
    end.line, end.column,
  );

  tasks.work(description, |tasks| match lazy.rget(expr) {
    Expression::Block(block) => verify_block(lazy, block, ret_ty, tasks),
    Expression::Literal { out, .. } => ty::verify_type(lazy, out, tasks),
    Expression::Variable { reference, .. } => verify_variable(lazy, *reference, tasks),
    Expression::Unknown { qualified, .. } => {
      // SPONGE: there must be a better way.
      let module = lazy.rget(expr.0.0).parent;
      let module_name = lazy.describe_module(module);

      tasks.seed_error(ErrorBase::UnknownTypeName {
        module_name,
        span: qualified.span,
      })
    },
    Expression::Unary { .. } => todo!(),
    Expression::Binary {
      a,
      b,
      op: (BinaryOperator::Assign, op_span),
      out,
      ..
    } => {
      let ty_reference = TypeReference::Expression(expr);
      let out_pair: TypePair = TypePair::new(ty_reference, out.clone());

      verify_expr(lazy, *a, None, tasks)?;
      verify_expr(lazy, *b, None, tasks)?;

      out_pair.coerce(lazy, &Type::Intrinsic {
        kind: Intrinsic::Void,
        span: *op_span,
      }, tasks)?;

      Ok(())
    },
    Expression::Binary { .. } => todo!(),
  })
}
