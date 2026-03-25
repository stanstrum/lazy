use crate::lang::expr::Expression;
use crate::lang::reference::{BlockReference, ExpressionReference, Store, TypeReference};
use crate::lang::span::GetSpan;
use crate::line_dbg;
use crate::resolve::coerce::{Coerce, SpecialPair, TypePair};
use crate::resolve::task_work;
use crate::resolve::verify::ty::verify_type;
use crate::tokenize::token::Span;

use super::*;

fn verify_expr(lazy: &Lazy, expr: ExpressionReference, ret_ty: &TypePair, tasks: &mut Tasks) -> Result<()> {
  let Span { start, end , .. } = lazy.rget(expr).get_span(lazy);

  task_work(tasks, line_dbg!("Verify expr").into(), |tasks| match lazy.rget(expr) {
    Expression::Block(block_reference) => todo!(),
    Expression::Literal { value, span, out } => verify_type(lazy, out),
    Expression::Variable { reference, span } => todo!(),
    Expression::Unknown { qualified, out } => todo!(),
    Expression::Unary { expr, op, span, out } => todo!(),
    Expression::Binary { a, b, op, span, out } => todo!(),
  })
}

pub(super) fn verify_block(lazy: &Lazy, block: &BlockReference, ret_ty: &TypePair, tasks: &mut Tasks) -> Result<()> {
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

    block_out.coerce(lazy, ret_ty, tasks)?;

    let last = block_borrow.returns_last.then(|| *block_borrow.children.last().unwrap());
    let is_last = |id: &_| last.is_some_and(|x| x == *id);

    for id in block_borrow.children.iter() {
      let expr = ExpressionReference(block.0, *id);

      let irr_reference = TypeReference::Expression(expr);
      let irr_ty = irr_reference.rget_from(lazy);

      let irr = SpecialPair(&irr_reference, irr_ty);

      if is_last(id) {
        irr.coerce(lazy, ret_ty, tasks)?;
      };

      verify_expr(lazy, expr, ret_ty, tasks)?;
    };

    Ok(())
  })
}
