use crate::aster::pprint::{Pretty, PrettyFunction};

use crate::lang::expr::Expression;
use crate::lang::expr::operator::BinaryOperator;
use crate::lang::reference::{BlockReference, Store, VariableReference};
use crate::lang::span::GetSpan;
use crate::line_dbg;

use crate::lang::Lazy;
use crate::lang::ty::{Intrinsic, Type};
use crate::lang::reference::{AliasReference, ExpressionReference, FunctionReference, ModuleReference, Reference, TypeReference};
use crate::resolve::coerce::TypePair;
use crate::resolve::task_work;
use crate::resolve::tasks::{OverwriteExpression, ResolveAsTask};
use crate::resolve::type_of::TypeOf;
use crate::resolve::coerce::{Coerce, SpecialPair};

use super::{Result, Error, Resolve, Tasks};

impl Resolve for AliasReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let reference = TypeReference::Alias(*self);
    let ty = &self.rget_from(lazy).ty;

    SpecialPair(&reference, ty).resolve(lazy, tasks)
  }
}

impl Resolve for ExpressionReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let description = {
      format!("Resolve ExpressionReference: {} in {}",
        self.print(lazy),
        self.0.print(lazy),
      )
    };

    task_work(tasks, description, |tasks| {
      let borrow = self.rget_from(lazy);
        let ty_reference = TypeReference::Expression(*self);

      match borrow {
        Expression::Literal { out, .. } => {
          SpecialPair(&ty_reference, out).resolve(lazy, tasks)
        },
        Expression::Variable { reference, .. } => {
          reference.resolve(lazy, tasks)
        },
        Expression::Binary { a, b, op: (BinaryOperator::Assign, op_span), span, out } => {
          let out_pair = SpecialPair(&ty_reference, out);

          let void_op = Type::Intrinsic {
            kind: Intrinsic::Void,
            span: *op_span,
          };

          out_pair.coerce(lazy, &void_op, tasks)?;
          out_pair.resolve(lazy, tasks)?;

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
              tasks.push(OverwriteExpression {
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

    task_work(tasks, description, |tasks| {
      TypeReference::Variable(*self).resolve(lazy, tasks)
    })
  }
}

impl Resolve for BlockReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let description = format!("Resolve BlockReference: {}", self.print(lazy));

    task_work(tasks, description, |tasks| {
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

impl Resolve for FunctionReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let description = format!(line_dbg!("Resolve FunctionReference: {}"), self.print(lazy));

    task_work(tasks, description, |tasks|{
      let function = self.rget_from(lazy);
      let ret_ty = TypeReference::ReturnTypeOf(*self);

      ret_ty.resolve(lazy, tasks)?;

      let arguments_iter = (0..function.header.arguments.len())
        .map(|index| TypeReference::Variable(VariableReference::Argument(*self, index)));

      for argument in arguments_iter {
        argument.resolve(lazy, tasks)?;
      };

      let body = lazy.rget(function.body);
      if let Some(ty) = ret_ty.type_of(lazy)? {
        let expr_id = body.children.last().unwrap();
        let reference = TypeReference::Expression(ExpressionReference(function.body, *expr_id));
        let typed_reference = Type::Reference(reference);

        let last_expression = SpecialPair(&reference, &typed_reference);
        let return_type: TypePair = SpecialPair(&ret_ty, &ty);

        last_expression.coerce(lazy, &return_type, tasks)?;
      };

      function.body.resolve(lazy, tasks)?;

      Ok(())
    })
  }
}

impl Resolve for ModuleReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let module = self.rget_from(lazy);

    for module in module.modules.iter() {
      module.resolve(lazy, tasks)?;
    };

    for function in module.functions.iter() {
      function.resolve(lazy, tasks)?;
    };

    for index in 0..module.aliases.len() {
      let reference = AliasReference(*self, index);
      reference.resolve(lazy, tasks)?;
    };

    Ok(())
  }
}
