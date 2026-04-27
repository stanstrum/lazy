use lazy_macros::print_once_per_thread;
use lang::{Compiler, CompilerPoolStore};
use lang::span::Span;
use lang::intrinsic::Intrinsic;
use lang::ty::{TypeKind, Type};
use lang::expr::Expression;
use lang::expr::operator::BinaryOperator;
use lang::reference::{BlockReference, ExpressionReference, TypeReference, VariableReference};
use pprint::Pretty;

use super::*;

impl<C: Compiler + 'static> Resolve<C> for VariableReference<C> {
  fn resolve(&self, store: &C::Store<'_>, tasks: &mut Tasks<C>) -> Result<C> {
    let description = {
      let (function, print): (_, String) = match self {
        VariableReference::Block(block_reference, _) => (block_reference.0, block_reference.print(store)),
        VariableReference::Argument(function_reference, _) => (*function_reference, pprint::print_function_reference::<C>(function_reference, store)),
      };

      format!(
        line_dbg!("Resolve VariableReference: {} in {}"),
        pprint::print_function_reference::<C>(&function, store),
        print,
      )
    };

    tasks.work(description, |tasks| {
      TypeReference::Variable(*self).resolve(store, tasks)
    })
  }
}

impl<C: Compiler + 'static> Coerce<C> for VariableReference<C> {
  fn coerce(&self, store: &C::Store<'_>, other: &impl TypeOf<C>, tasks: &mut Tasks<C>) -> Result<C> {
    TypeReference::Variable(*self).coerce(store, other, tasks)
  }
}

fn verify_variable<C: Compiler + 'static>(store: &C::Store<'_>, variable: VariableReference<C>, tasks: &mut Tasks<C>) -> Result<C> {
  let description = {
    let (function, print): (_, String) = match &variable {
      VariableReference::Block(block_reference, _) => (block_reference.0, block_reference.print(store)),
      VariableReference::Argument(function_reference, _) => (*function_reference, pprint::print_function_reference::<C>(function_reference, store)),
    };

    let function_borrow = store.rget(function);
    let parent = function_borrow.parent;

    format!(
      line_dbg!("Verify variable: {}::{} in {}"),
      store.describe_module(parent),
      function_borrow.header.name.print(store),
      print,
    )
  };

  tasks.work(description, |tasks| {
    ty::verify_typeof(store, &TypeReference::Variable(variable), tasks)
  })
}

impl<C: Compiler + 'static> Coerce<C> for ExpressionReference<C> {
  fn coerce(&self, store: &C::Store<'_>, other: &impl TypeOf<C>, tasks: &mut Tasks<C>) -> Result<C> {
    let a = self.print(store);
    let b = other.type_of(store).map(|x| x.print(store)).unwrap_or_else(|| "{none}".into());

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

      TypeReference::Expression(*self).coerce(store, other, tasks)
    })
  }
}

impl<C: Compiler + 'static> Resolve<C> for BlockReference<C> {
  fn resolve(&self, store: &C::Store<'_>, tasks: &mut Tasks<C>) -> Result<C> {
    let description = format!(line_dbg!("Resolve BlockReference: {}"), self.print(store));

    tasks.work(description, |tasks| {
      let borrow = self.rget_from(store);

      for id in 0..borrow.variables.len() {
        VariableReference::Block(*self, id).resolve(store, tasks)?;
      };

      for &expr in borrow.children.iter() {
        ExpressionReference(*self, expr).resolve(store, tasks)?;
      };

      Ok(())
    })
  }
}

pub(super) fn default_types_in_block_expr<C: Compiler + 'static>(store: &mut C::Store<'_>, block: &BlockReference<C>, tasks: &mut Tasks<C>) -> Result<C> {
  let description = {
    let Span { start, end, .. } = block.get_span(store);

    format!(line_dbg!("Make default ambiguous types for block: {}:{} - {}:{}"),
      start.line, start.column,
      end.line, end.column,
    )
  };

  tasks.work(description, |tasks| {
    ty::default_types_of_type(store, &TypeReference::Block(*block), tasks)?;

    for expr in block.rget_from(store).children.clone() {
      default_types_in_expr(store, &ExpressionReference(*block, expr), tasks)?;
    };

    Ok(())
  })
}

pub(super) fn verify_block<C: Compiler + 'static>(store: &C::Store<'_>, block: &BlockReference<C>, ret_ty: Option<&Type<C>>, tasks: &mut Tasks<C>) -> Result<C> {
  let block_borrow = block.rget_from(store);

  let description = {
    let Span { start, end, .. } = block_borrow.span;

    format!(line_dbg!("Verify block: {}:{} - {}:{}"),
      start.line, start.column,
      end.line, end.column,
    )
  };

  tasks.work(description, |tasks| {
    let block_type_reference = TypeReference::Block(*block);
    let block_out = Type::new(block_type_reference, block_borrow.out.clone());

    if let Some(ret_ty) = ret_ty {
      block_out.coerce(store, ret_ty, tasks)?;
    };

    let last = block_borrow.returns_last.then(|| *block_borrow.children.last().unwrap());
    let is_last = |id: &_| last.is_some_and(|x| x == *id);

    for id in block_borrow.children.iter() {
      let expr = ExpressionReference(*block, *id);

      let irr_reference = TypeReference::Expression(expr);
      let irr_ty = irr_reference.rget_from(store);

      let irr = Type::new(irr_reference, irr_ty.clone());

      let ret_ty = if is_last(id) && let Some(ret_ty) = ret_ty {
        irr.coerce(store, ret_ty, tasks)?;
        Some(ret_ty)
      } else {
        None
      };

      verify_expr(store, expr, ret_ty, tasks)?;
    };

    Ok(())
  })
}

impl<C: Compiler + 'static> Resolve<C> for ExpressionReference<C> {
  fn resolve(&self, store: &C::Store<'_>, tasks: &mut Tasks<C>) -> Result<C> {
    let description = {
      format!(line_dbg!("Resolve ExpressionReference: {} in {}"),
        self.print(store),
        self.0.print(store),
      )
    };

    tasks.work(description, |tasks| {
      let borrow = self.rget_from(store);
        let ty_reference = TypeReference::Expression(*self);

      match borrow {
        Expression::Block(block) => {
          block.resolve(store, tasks)
        },
        Expression::Literal { out, .. } => {
          Type::new(ty_reference, out.clone()).resolve(store, tasks)
        },
        Expression::Variable { reference, .. } => {
          reference.resolve(store, tasks)
        },
        Expression::Binary {
          a, b,
          op: (BinaryOperator::Assign, op_span),
          out,
          ..
        } => {
          let out_pair = Type::new(ty_reference, out.clone());

          let void_op = TypeKind::Intrinsic {
            kind: Intrinsic::Void,
            span: *op_span,
          };

          out_pair.coerce(store, &void_op, tasks)?;
          out_pair.resolve(store, tasks)?;

          a.coerce(store, b, tasks)?;
          b.coerce(store, a, tasks)?;

          a.resolve(store, tasks)?;
          b.resolve(store, tasks)?;

          Ok(())
        },
        Expression::Unknown { qualified, .. } if qualified.parts.len() == 1 && !qualified.is_implicit() => {
          let part = qualified.parts.first().unwrap();

          let mut block = Some(self.0);

          let hierarchy = std::iter::from_fn(move || {
            let old = block;
            block = block.and_then(|block| store.rget(block).parent);

            old
          });

          let variables_iter_iter = hierarchy.map(|block| {
            store.rget(block)
              .variables.iter().enumerate()
              // I do not understand why this lambda is `move` ...
              .map(move |(id, var)| (var.name.id, VariableReference::Block(block, id))
            )
          });

          let function = self.0.0;
          let arguments_iter = store.rget(function)
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
                  span: borrow.get_span(store),
                },
              }, line_dbg!("here"));

              return Ok(());
            };
          };

          tasks.seed_error(ResolveErrorBase::UnknownTypeName {
            module_name: store.describe_module(self.0.0.rget_from(store).parent),
            span: borrow.get_span(store),
          })
        },
        Expression::StructInitializer { ty, members, .. } => {
          let prototype = ty.type_of(store).map(|ty| {
            let TypeKind::Struct { prototype } = ty else {
              todo!("error for bad struct initializer type at resolve");
            };

            prototype
          });

          Type::new(
            TypeReference::Expression(*self),
            ty.clone(),
          ).resolve(store, tasks)?;

          print_once_per_thread!(store, {
            level: Stub,
            force: false,
            description: line_dbg!("coerce member expressions from struct `ty`").into(),
            contents: MessageContents::WithinSource(
              WithinSource::new(
                members.iter().map(|(name, expr)| {
                  let span = Span::from_pair(name.span, expr.get_span(store));

                  log::MessageSection {
                    text: "here".into(),
                    span,
                  }
                }).collect(),
              ),
            ),
          });

          for (member_name, member_expr) in members.iter() {
            member_expr.resolve(store, tasks)?;

            if let Some(prototype) = prototype {
              let field_ty = store.rget(prototype).members.iter()
                .enumerate()
                .find_map(|(id, field)| (field.name.id == member_name.id).then_some(TypeReference::StructMember(prototype, id)))
                .expect("to find a corresponding field for a struct initializer member");

              member_expr.coerce(store, &field_ty, tasks)?;
              field_ty.coerce(store, &TypeReference::Expression(*member_expr), tasks)?;
            };
          };

          Ok(())
        },
        other => tasks.seed_error(ResolveErrorBase::NotImplemented {
          what: line_dbg!("impl Resolve for ExpressionReference"),
          span: other.get_span(store),
        }),
      }
    })
  }
}

fn default_types_in_expr<C: Compiler + 'static>(store: &mut C::Store<'_>, expr: &ExpressionReference<C>, tasks: &mut Tasks<C>) -> Result<C> {
  let Span { start, end , .. } = expr.get_span(store);

  let description = format!(line_dbg!("Make default ambiguous types for expr {}:{} - {}:{}"),
    start.line, start.column,
    end.line, end.column,
  );

  ty::default_types_of_type(store, &TypeReference::Expression(*expr), tasks)?;

  tasks.work(description, |tasks| {
    match expr.rget_from(store) {
      &Expression::Block(block) => {
        default_types_in_block_expr(store, &block, tasks)?;
      },
      Expression::Literal { .. } => {},
      Expression::Variable {.. } => {},
      Expression::Unknown { .. } => todo!(),
      Expression::Unary { expr: a, .. } => {
        // SPONGE: There are actually two type fields in a unary expression because
        // one is contained within the expr and one is part of the Expression
        // variant ... maybe fix this?
        ty::default_types_of_type(store, &TypeReference::Expression(*a), tasks)?;
      },
      &Expression::Binary { a, b, .. } => {
        default_types_in_expr(store, &a, tasks)?;
        default_types_in_expr(store, &b, tasks)?;
      },
      Expression::StructInitializer { members, .. } => {
        let value_iter = members.iter()
          .map(|(_, value)| *value)
          .collect::<Vec<_>>();

        for value in value_iter {
          default_types_in_expr(store, &value, tasks)?;
        };
      },
    };

    ty::default_types_of_type(store, &TypeReference::Expression(*expr), tasks)
  })
}

fn verify_expr<C: Compiler + 'static>(store: &C::Store<'_>, expr: ExpressionReference<C>, ret_ty: Option<&Type<C>>, tasks: &mut Tasks<C>) -> Result<C> {
  let Span { start, end , .. } = store.rget(expr).get_span(store);

  let description = format!(line_dbg!("Verify expr {}:{} - {}:{}"),
    start.line, start.column,
    end.line, end.column,
  );

  tasks.work(description, |tasks| match store.rget(expr) {
    Expression::Block(block) => verify_block(store, block, ret_ty, tasks),
    Expression::Literal { out, .. } => ty::verify_type(store, out, tasks),
    Expression::Variable { reference, .. } => verify_variable(store, *reference, tasks),
    Expression::Unknown { qualified, .. } => {
      // SPONGE: there must be a better way.
      let module = store.rget(expr.0.0).parent;
      let module_name = store.describe_module(module);

      tasks.seed_error(ResolveErrorBase::UnknownTypeName {
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
      let out_pair: Type<C> = Type::new(ty_reference, out.clone());

      verify_expr(store, *a, None, tasks)?;
      verify_expr(store, *b, None, tasks)?;

      // TypeReference::Expression(*a).coerce(lazy, &TypeReference::Expression(*b), tasks)?;
      // TypeReference::Expression(*b).coerce(lazy, &TypeReference::Expression(*a), tasks)?;

      out_pair.coerce(store, &TypeKind::Intrinsic {
        kind: Intrinsic::Void,
        span: *op_span,
      }, tasks)?;

      Ok(())
    },
    Expression::Binary { .. } => todo!(),
    Expression::StructInitializer { ty, members, .. } => {
      let Some(prototype) = ty.type_of(store).map(|ty| {
        let TypeKind::Struct { prototype } = ty else {
          todo!("error for bad struct initializer type at resolve");
        };

        prototype
      }) else {
        todo!("error for uninitialized struct initializer type");
      };

      for (struct_index, field) in store.rget(prototype).members.iter().enumerate() {
        let expr_reference = members.iter()
          .find_map(|(name, value)| (field.name.id == name.id).then_some(value))
          .expect("to find a corresponding field for this struct initializer member");

        let Some(ty) = field.ty.type_of(store) else {
          todo!("unresolved type");
        };

        let ret_ty = Type::new(
          TypeReference::StructMember(prototype, struct_index),
          ty,
        );

        verify_expr(store, *expr_reference, Some(&ret_ty), tasks)?;
      };

      Ok(())
    },
  })
}
