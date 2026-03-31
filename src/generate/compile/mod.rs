mod expr;

use super::*;

fn compile_function(comp: &mut Compilation, function: lang::reference::FunctionReference) -> Result {
  let borrow = comp.lazy.rget(function);
  let body = borrow.body;

  let function_value = comp.get_or_declare_function(function)?;

  let return_last = expr::compile_block(comp, function_value, body)?;

  todo!()
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
