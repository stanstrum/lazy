use super::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct VariableEntry<C: Compiler> {
  name_id: string_pool::PoolId,
  reference: lang::reference::VariableReference<C>,
}

pub(crate) fn iter_block_variables<'store, C: Compiler + 'static>(
  store: &'store C::Store<'_>,
  block_reference: BlockReference<C>,
) -> impl Iterator<Item = VariableEntry<C>> {
  let function_reference = block_reference.0;

  let function_borrow = store.rget(function_reference);
  let arg_iters = function_borrow.header.arguments
    .iter().enumerate()
    .map(move |(index, variable)| {
      let name_id = variable.name.id;
      let reference = lang::reference::VariableReference::Argument(function_reference, index);

      VariableEntry { name_id, reference }
    });

  let mut block_reference = Some(block_reference);
  let entry_iters = std::iter::from_fn(move || {
    let curr_block_reference = block_reference?;
    let block_borrow = curr_block_reference.rget_from(store);

    let iter = block_borrow.variables.iter().enumerate()
      .map(move |(index, variable)| {
        let name_id = variable.name.id;
        let reference = lang::reference::VariableReference::Block(curr_block_reference, index);

        VariableEntry { name_id, reference }
      });

    block_reference = block_borrow.parent;

    Some(iter)
  });

  // NOTE: Make sure arg_iters comes last since block variables take precedence
  entry_iters.flatten().chain(arg_iters)
}

pub(crate) fn resolve_variable<C: Compiler + 'static>(
  store: &C::Store<'_>,
  block_reference: BlockReference<C>,
  id: &string_pool::PoolId,
) -> Option<lang::reference::VariableReference<C>> {
  iter_block_variables(store, block_reference).find_map(|entry| {
    (&entry.name_id == id).then_some(entry.reference)
  })
}
