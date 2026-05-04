use super::*;

pub(crate) fn typify_function_reference<'store, C: Compiler + 'static>(resolver: &'store Resolver<'store, '_, '_, C>, function_reference: C::FunctionReference) -> Box<dyn Iterator<Item = TypeReference<C>> + 'store> {
  let borrow = function_reference.rget_from(resolver.store);
  let args = (0..borrow.header.arguments.len())
    .map(move |index| TypeReference::Variable(
      VariableReference::Argument(function_reference, index)
    )
  );
  let return_type = std::iter::once(TypeReference::ReturnTypeOf(function_reference));

  let exprs = borrow.body.get_type_iter(resolver.store);

  Box::new(return_type.chain(args).chain(exprs))
}
