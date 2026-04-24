use super::*;

pub(super) fn compile_variable<'ctx>(
  _comp: &mut Compilation<'_, '_, 'ctx>,
  variable: &lang::VariableReference,
  scopes: &mut FunctionScopes<'ctx>,
) -> Result<LazyValue<'ctx>> {
  match variable {
    lang::VariableReference::Block(block_to_find, index) => {
      let Some(scope) = scopes.scopes.iter().rfind(|scope| {
        &scope.block == block_to_find
      }) else {
        panic!("invalid variable reference");
      };

      Ok(LazyValue::Pointer({
        scope.variables.get(*index).unwrap()
          .to_owned()
      }))
    },
    lang::VariableReference::Argument(_, index) => Ok({
      scopes.value.get_nth_param(*index as _)
        .expect("to get nth param")
        .into()
    }),
  }
}
