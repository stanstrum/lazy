mod expr;

use super::*;

fn compile_function(comp: &mut Compilation, function: lang::reference::FunctionReference) -> Result {
  let borrow = comp.lazy.rget(function);
  let body = borrow.body;

  let function_value = comp.get_or_declare_function(function)?;

  let last_value = expr::compile_block(comp, function_value, body)?
    .as_basic_value_enum()
    .ok();

  let last_value = last_value.as_ref()
    .map(|value| value as _);

  comp.llvm.builder.build_return(last_value)
    .expect("build_return");

  Ok(())
}

pub(super) fn compile_module(comp: &mut Compilation, module: lang::reference::ModuleReference) -> Result {
  for &module in comp.lazy.rget(module).modules.iter() {
    compile_module(comp, module)?;
  };

  for &function in comp.lazy.rget(module).functions.iter() {
    compile_function(comp, function)?;
  };

  Ok(())
}
