pub(crate) mod block;

use super::*;

impl<C: Compiler + 'static> Resolve<C> for VariableReference<C> {
  fn resolve(&self, resolver: &Resolver<C>) -> Result<C, bool> {
    self.rget_from(resolver.store).ty.resolve(resolver)
  }
}

impl<C: Compiler + 'static> Typify<C> for ExpressionReference<C> {
  fn get_type_iter<'store>(self, store: &'store C::Store<'_>) -> Box<dyn Iterator<Item = Box<dyn Resolve<C>>> + 'store> {
    match store.rget(self) {
      lang::expr::Expression::Block(block_reference) => Box::new(block_reference.get_type_iter(store)),
      lang::expr::Expression::Literal { out, .. } => {
        let Some(a) = out.type_of(store) else {
          todo!();
        };

        let a: Box<dyn Resolve<C>> = Box::new(a);
        let a = std::iter::once(a);
        let a = Box::new(a);

        a
      },
      lang::expr::Expression::Variable { reference, .. } => {
        // compiler doesn't infer this
        let ugh: Box<dyn Resolve<C>> = Box::new(*reference);

        Box::new(std::iter::once(ugh))
      },
      lang::expr::Expression::Binary { a, b, .. } => {
        let a = a.get_type_iter(store);
        let b = b.get_type_iter(store);

        Box::new(a.chain(b))
      },
      other => todo!("{other:#?}"),
    }
  }
}
