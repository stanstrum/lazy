use crate::aster::pprint::Pretty;
use crate::lang::expr::Expression;
use crate::lang::expr::operator::BinaryOperator;
use crate::lang::reference::{BlockReference, ExpressionReference, Store, TypeReference, VariableReference};
use crate::lang::span::GetSpan;
use crate::lang::ty::{Intrinsic, Type};
use crate::line_dbg;
use crate::resolve::coerce::{Coerce, SpecialPair, TypePair};
use crate::resolve::task_work;
use crate::resolve::verify::ty::{verify_type, verify_typeof};
use crate::tokenize::token::Span;

use super::*;

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

  task_work(tasks, description, |_| {
    verify_typeof(lazy, &TypeReference::Variable(variable))
  })
}

fn verify_expr(lazy: &Lazy, expr: ExpressionReference, ret_ty: Option<&TypePair>, tasks: &mut Tasks) -> Result<()> {
  let Span { start, end , .. } = lazy.rget(expr).get_span(lazy);

  let description = format!(line_dbg!("Verify expr {}:{} - {}:{}"),
    start.line, start.column,
    end.line, end.column,
  );

  task_work(tasks, description, |tasks| match lazy.rget(expr) {
    Expression::Block(block) => verify_block(lazy, block, None, tasks),
    Expression::Literal { out, .. } => verify_type(lazy, out),
    Expression::Variable { reference, .. } => verify_variable(lazy, *reference, tasks),
    Expression::Unknown { qualified, .. } => {
      // SPONGE: there must be a better way.
      let module = lazy.rget(expr.0.0).parent;
      let module_name = lazy.describe_module(module);

      Err(Box::new(Error::UnknownTypeName {
        module_name,
        span: qualified.span,
      }))
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
      let out_pair: TypePair = SpecialPair(&ty_reference, out);

      verify_expr(lazy, *a, None, tasks)?;
      verify_expr(lazy, *b, None, tasks)?;

      out_pair.coerce(lazy, &Type::Intrinsic {
        kind: Intrinsic::Void,
        span: *op_span,
      }, tasks)?;

      todo!()
    },
    Expression::Binary { .. } => todo!(),
  })
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

  task_work(tasks, description, |tasks| {
    let block_type_reference = TypeReference::Block(*block);
    let block_out = SpecialPair(&block_type_reference, &block_borrow.out);

    if let Some(ret_ty) = ret_ty {
      block_out.coerce(lazy, ret_ty, tasks)?;
    };

    let last = block_borrow.returns_last.then(|| *block_borrow.children.last().unwrap());
    let is_last = |id: &_| last.is_some_and(|x| x == *id);

    for id in block_borrow.children.iter() {
      let expr = ExpressionReference(*block, *id);

      let irr_reference = TypeReference::Expression(expr);
      let irr_ty = irr_reference.rget_from(lazy);

      let irr = SpecialPair(&irr_reference, irr_ty);

      if is_last(id) && let Some(ret_ty) = ret_ty {
        irr.coerce(lazy, ret_ty, tasks)?;
      };

      verify_expr(lazy, expr, ret_ty, tasks)?;
    };

    Ok(())
  })
}
