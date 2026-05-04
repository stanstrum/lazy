use super::*;

impl<C: Compiler + 'static> Typify<C> for BlockReference<C> {
  fn get_type_iter<'store>(self, store: &'store C::Store<'_>) -> Box<dyn Iterator<Item = TypeReference<C>> + 'store> {
    Box::new(store.rget(self).children.iter().map(move |expr_id| {
      let expression_reference = ExpressionReference(self, *expr_id);
      expression_reference.get_type_iter(store)
    }).flatten())
  }
}
