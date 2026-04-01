use crate::generate::types::make_type;

use super::*;

pub(super) fn compile_literal<'ctx>(
  comp: &mut Compilation<'_, '_, 'ctx>,
  _function: inkwell::values::FunctionValue<'ctx>,
  kind: lang::expr::LiteralKind,
  out: &lang::ty::Type,
) -> Result<LazyValue<'ctx>> {
  match kind {
    lang::expr::LiteralKind::Numeric(token::NumericValue::U64(value)) => {
      // we use `out` because, despite internally storing u64, the literal
      // could be u32, i32, etc.
      let value = make_type(comp, out)?
        .into_int_type()?
        // SPONGE: `sign_extend` might be used for something
        .const_int(value, false);

      Ok(LazyValue::Int(value))
    },
    lang::expr::LiteralKind::Numeric(token::NumericValue::F64(_)) => todo!(),
    lang::expr::LiteralKind::String { .. } => todo!(),
  }
}
