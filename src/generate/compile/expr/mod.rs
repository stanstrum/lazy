mod literal;
mod variable;

use crate::lang::expr::operator::BinaryOperator;
use crate::generate::types::LazyValue;

use super::*;

fn compile_expr<'ctx>(
  comp: &mut Compilation<'_, '_, 'ctx>,
  function: inkwell::values::FunctionValue<'ctx>,
  expr: lang::reference::ExpressionReference,
  scopes: &mut FunctionScopes<'ctx>,
) -> Result<LazyValue<'ctx>> {
  // for efficiency, we assume we are already positioned after the preceding
  // instruction.  otherwise we'd need block: BlockValue<'_> passed in, and then
  // call:
  //
  // comp.llvm.builder.position_at_end(block);

  match comp.lazy.rget(expr) {
    &lang::expr::Expression::Block(block)
      => compile_block(comp, function, block, scopes),
    &lang::expr::Expression::Literal { value, ref out, .. }
      => literal::compile_literal(comp, function, value, out),
    lang::expr::Expression::Variable { reference, .. }
      => variable::compile_variable(comp, reference, scopes),
    lang::expr::Expression::Unknown { .. } => todo!(),
    lang::expr::Expression::Unary { .. } => todo!(),
    lang::expr::Expression::Binary { a, b, op: (BinaryOperator::Assign, _), .. } => {
      let lhs = compile_expr(comp, function, *a, scopes)?;
      let rhs = compile_expr(comp, function, *b, scopes)?;

      let ptr = lhs.as_basic_value_enum()
        .expect("lhs value to to be PointerValue")
        .into_pointer_value();
      let value  = rhs.as_basic_value_enum()
        .expect("rhs to be BasicValueEnum");

      comp.llvm.builder.build_store(ptr, value)?;

      Ok(LazyValue::Void)
    },
    other => todo!("{other:#?}"),
  }
}

pub(super) fn compile_block<'ctx>(
  comp: &mut Compilation<'_, '_, 'ctx>,
  function: inkwell::values::FunctionValue<'ctx>,
  block: lang::reference::BlockReference,
  scopes: &mut FunctionScopes<'ctx>,
) -> Result<LazyValue<'ctx>> {
  // Note where we came from -- will need to jmp from that block to this one
  let prev_block = comp.llvm.builder.get_insert_block()
    .expect("to have come from a previous BasicBlock");

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

  // write out our variables for this scope
  scopes.push(comp, block)?;

  // position the builder at the label so we can insert from there on
  comp.llvm.builder.position_at_end(here);

  let mut last_value = LazyValue::Void;

  let borrow = comp.lazy.rget(block);

  for &id in borrow.children.iter() {
    let expr = lang::reference::ExpressionReference(block, id);
    let value = compile_expr(comp, function, expr, scopes)?;

    last_value = value;
  };

  // !!! do not forget to do this !!!
  // SPONGE: i could mandate this with #[must_use] and Drop
  scopes.pop();

  // Build the joins between this block and the last
  let after_prev_block = here;

  let continue_block = comp.llvm.builder.get_insert_block()
    .expect("to be positioned in a BasicBlock");

  assert!(prev_block != after_prev_block, "prev_block and after_prev_block are the same!");

  // Position to where we came from
  comp.llvm.builder.position_at_end(prev_block);
  // Build a branch from thence to block we just
  comp.llvm.builder.build_unconditional_branch(after_prev_block)?;
  comp.llvm.builder.position_at_end(continue_block);

  Ok(last_value)
}
