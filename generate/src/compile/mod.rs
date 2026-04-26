mod expr;

use gluezy::LazyStructures;

use crate::types::make_type;

use super::*;

struct FunctionScope<'ctx> {
  block: lang::reference::BlockReference<LazyStructures>,
  variables: Vec<inkwell::values::PointerValue<'ctx>>,
}

struct FunctionScopes<'ctx> {
  reference: gluezy::FunctionReference,
  value: inkwell::values::FunctionValue<'ctx>,
  scopes: Vec<FunctionScope<'ctx>>,
}

impl<'ctx> FunctionScopes<'ctx> {
  fn new(
    reference: gluezy::FunctionReference,
    value: inkwell::values::FunctionValue<'ctx>,
  ) -> Self {
    Self {
      reference,
      value,
      scopes: vec![],
    }
  }

  fn push(&mut self, comp: &mut Compilation<'_, '_, 'ctx>, block: lang::reference::BlockReference<LazyStructures>) -> Result {
    let borrow = comp.lazy.rget(block);

    let variables = borrow.variables.iter()
      .map(|variable| {
        let name = comp.lazy.pool.get(variable.name.id);
        let ty = make_type(comp, &variable.ty)?;

        let pointer = comp.llvm.builder.build_alloca(
          ty.as_basic_type_enum().expect("type to be BasicValueEnum"),
          &name
        )?;

        Ok(pointer)
      }).collect::<Result<Vec<_>>>()?;

    self.scopes.push(FunctionScope {
      block,
      variables,
    });

    Ok(())
  }

  fn pop(&mut self) {
    self.scopes.pop();
  }
}

fn compile_function(comp: &mut Compilation, function_reference: gluezy::FunctionReference) -> Result {
  let borrow = comp.lazy.rget(function_reference);
  let body = borrow.body;

  let function_value = comp.get_or_declare_function(function_reference)?;

  let entry = function_value.get_last_basic_block()
    .expect("function to have an entry block");

  assert!(entry.get_name().to_string_lossy() == "entry");
  comp.llvm.builder.position_at_end(entry);

  let mut scopes = FunctionScopes::new(function_reference, function_value);

  let last_value = expr::compile_block(comp, function_value, body, &mut scopes)?
    .as_basic_value_enum();

  let last_value = last_value.as_ref().map(|x| x as _);

  comp.llvm.builder.build_return(last_value)?;

  Ok(())
}

pub(super) fn compile_module(comp: &mut Compilation, module: gluezy::ModuleReference) -> Result {
  for &module in comp.lazy.rget(module).modules.iter() {
    compile_module(comp, module)?;
  };

  for &function in comp.lazy.rget(module).functions.iter() {
    compile_function(comp, function)?;
  };

  Ok(())
}
