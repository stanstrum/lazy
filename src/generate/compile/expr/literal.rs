use crate::generate::types::make_type;

use super::*;

pub(super) fn compile_literal<'ctx>(
  comp: &mut Compilation<'_, '_, 'ctx>,
  function: inkwell::values::FunctionValue<'ctx>,
  kind: lang::expr::LiteralKind,
  out: &lang::ty::Type,
) -> Result<inkwell::values::InstructionValue<'ctx>> {
  match kind {
    lang::expr::LiteralKind::Numeric(token::NumericValue::U64(value)) => {
      let value = make_type(comp, out)?
        .into_int_type()
        // SPONGE: `sign_extend` might be used for something
        .const_int(value, false);

      dbg!(&value);
      dbg!(value.as_instruction());

      todo!()
    },
    lang::expr::LiteralKind::Numeric(token::NumericValue::F64(value)) => todo!(),
    lang::expr::LiteralKind::Numeric(numeric_value) => todo!(),
    lang::expr::LiteralKind::String { value, kind } => todo!(),
  }
}
