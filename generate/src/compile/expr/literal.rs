use inkwell::values::BasicValue;

use gluezy::LazyStructures;
use lang::span::Span;
use lang::token;

use crate::types::make_type;

use super::*;

pub(super) fn compile_literal<'ctx>(
  comp: &mut Compilation<'_, '_, 'ctx>,
  function: inkwell::values::FunctionValue<'ctx>,
  kind: lang::expr::LiteralKind,
  out: &lang::ty::Type<LazyStructures>,
) -> Result<LazyStructures, LazyValue<'ctx>> {
  match kind {
    lang::expr::LiteralKind::Numeric(token::NumericValue::U64(value)) => {
      // we use `out` because, despite internally storing u64, the literal
      // could be u32, i32, etc.
      let value = make_type(comp, out)?
        .into_int_type().expect("literal out type to be IntType")
        // SPONGE: `sign_extend` might be used for something
        .const_int(value, false);

      Ok(LazyValue::Int(value))
    },
    lang::expr::LiteralKind::Numeric(token::NumericValue::F64(_)) => todo!(),
    lang::expr::LiteralKind::String { kind, value }
      => compile_string_literal(comp, function, kind, value, out),
  }
}

fn compile_string_literal<'ctx>(
  comp: &mut Compilation<'_, '_, 'ctx>,
  _function: inkwell::values::FunctionValue<'ctx>,
  kind: token::StringKind,
  value: string_pool::StringId,
  out: &lang::ty::Type<LazyStructures>,
) -> Result<LazyStructures, LazyValue<'ctx>> {
  // SAFETY: this ref doesn't leave scope and we aren't borrowing anything else
  //         while we're using it
  let string_ref = unsafe { comp.lazy.pool.get_string(value) };

  let (element_type, values);
  let prefix = match kind {
    token::StringKind::Wide => {
      // This is the base type that we'll be constructing the global with.
      // it corresponds to the type of each individual character, which
      // in this case, is u32 (LLVM prefixes all ints with "i")
      element_type = comp.llvm.context.i32_type();

      // I believe there's a method that does this already, but it
      // should be useful later to have finer control over how this
      // constant is built
      values = string_ref.chars().map(|ch| {
        // SPONGE: sign extend ?
        element_type.const_int(ch as _, false)
      }).collect::<Vec<_>>();

      "wstrlit"
    },
    token::StringKind::Byte => {
      element_type = comp.llvm.context.i8_type();

      values = string_ref.chars().map(|ch| {
        element_type.const_int(ch as _, false)
      }).collect::<Vec<_>>();

      "bstrlit"
    },
    token::StringKind::C => {
      element_type = comp.llvm.context.i8_type();

      let value_iter = string_ref.chars().map(|ch| {
        element_type.const_int(ch as _, false)
      });

      let null_term_iter = std::iter::once(element_type.const_zero());
      values = value_iter.chain(null_term_iter).collect::<Vec<_>>();

      "cstrlit"
    },
  };

  // Make a ~unique~ish name for the literal for debugging purposes.
  // Not necessary; I believe LLVM or Inkwell adds a suffix if the name
  // is already taken.[citation needed]
  let Span { start, end, .. } = out.get_span(comp.lazy);
  let name = format!("{prefix}_from_{}_{}_to_{}_{}",
    start.line, start.column,
    end.line, end.column,
  );

  // Create the ArrayValue, note that we haven't actually done anything;
  // we have to store this inside of a global
  let array_value = element_type.const_array(&values);

  // This will be the type of the global itself
  let array_type = array_value.get_type();

  // So here we've created a GlobalValue in the LLVM module, but it has
  // no data.  In effect, this value corresponds to an empty declaration
  let global_value = comp.llvm.module.add_global(array_type, None, &name);

  // SPONGE: does not support static mut strings
  global_value.set_constant(true);

  // This finally stores the value we created earlier with the IntValues
  // into the global value
  global_value.set_initializer(&array_value.as_basic_value_enum());

  // Return the pointer of the GlobalValue, as that's how the value we
  // configured will be accessible
  Ok(LazyValue::Pointer(global_value.as_pointer_value()))
}
