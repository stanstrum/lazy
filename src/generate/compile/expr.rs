mod literal;

use crate::generate::types::LazyValue;

use super::*;

fn compile_expr<'ctx>(
  comp: &mut Compilation<'_, '_, 'ctx>,
  function: inkwell::values::FunctionValue<'ctx>,
  expr: lang::reference::ExpressionReference,
) -> Result<LazyValue<'ctx>> {
  // for efficiency, we assume we are already positioned after the preceding
  // instruction.  otherwise we'd need block: BlockValue<'_> passed in, and then
  // call:
  //
  // comp.llvm.builder.position_at_end(block);

  match comp.lazy.rget(expr) {
    &lang::expr::Expression::Block(block) => {
      let prev_block = comp.llvm.builder.get_insert_block()
        .expect("to have come from a previous BasicBlock");

      let value = compile_block(comp, function, block)?;

      let continue_position = comp.llvm.builder.get_insert_block()
        .expect("to be positioned in a BasicBlock");

      let after_prev_block = prev_block.get_next_basic_block()
        .expect("to have created a BasicBlock");

      assert!(prev_block != after_prev_block, "prev_block and after_prev_block are the same!");

      comp.llvm.builder.position_at_end(prev_block);
      comp.llvm.builder.build_unconditional_branch(after_prev_block)
        .expect("to create unconditional branch");
      comp.llvm.builder.position_at_end(continue_position);

      Ok(value)
    },
    &lang::expr::Expression::Literal { value, ref out, .. }
      => literal::compile_literal(comp, function, value, out),
    lang::expr::Expression::Variable { .. } => todo!(),
    lang::expr::Expression::Unknown { .. } => todo!(),
    lang::expr::Expression::Unary { .. } => todo!(),
    lang::expr::Expression::Binary { .. } => todo!(),
  }
}

pub(super) fn compile_block<'ctx>(
  comp: &mut Compilation<'_, '_, 'ctx>,
  function: inkwell::values::FunctionValue<'ctx>,
  block: lang::reference::BlockReference,
) -> Result<LazyValue<'ctx>> {
  // TODO: could the names be more imaginative here?
  //       perhaps programmatically named for clarity
  let span = block.get_span(comp.lazy);
  let name = format!("at_{}_{}_to_{}_{}",
    span.start.line,
    span.start.column,
    span.end.line,
    span.end.column,
  );

  // create the label we will use for this block
  let here = comp.llvm.context.append_basic_block(function, &name);

  // position the builder at the label so we can insert from there on
  comp.llvm.builder.position_at_end(here);

  let mut last_value = LazyValue::Void;

  let borrow = comp.lazy.rget(block);

  for &id in borrow.children.iter() {
    let expr = lang::reference::ExpressionReference(block, id);
    let value = compile_expr(comp, function, expr)?;

    last_value = value;
  };

  Ok(last_value)
}
