use super::*;

pub(crate) fn typify_function_reference<'store, C: Compiler + 'static>(
  resolver: &'store Resolver<'store, '_, '_, C>,
  function_reference: C::FunctionReference,
) -> Box<dyn Iterator<
  Item = Box<dyn Resolve<C>>> + 'store
> {
  let borrow = function_reference.rget_from(resolver.store);
  let args = (0..borrow.header.arguments.len())
    .map(move |index| {
      let m: Box<dyn Resolve<C>> = Box::new(TypeReference::Variable(
        VariableReference::Argument(function_reference, index)
      ));

      m
  });
  let return_type = TypeReference::ReturnTypeOf(function_reference);
  let return_type: Box<dyn Resolve<C>> = Box::new(return_type);
  let return_type = std::iter::once(return_type);

  let exprs = borrow.body.get_type_iter(resolver.store);

  let iter = return_type.chain(args).chain(exprs);

  Box::new(iter)
}
