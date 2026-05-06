pub(crate) mod block;

use super::*;

impl<C: Compiler + 'static> Typify<C> for ExpressionReference<C> {
  fn get_type_iter<'store>(self, store: &'store C::Store<'_>) -> Box<dyn Iterator<Item = Box<dyn Resolve<C>>> + 'store> {
    match store.rget(self) {
      lang::expr::Expression::Block(block_reference) => Box::new(block_reference.get_type_iter(store)),
      lang::expr::Expression::Literal { value, span, out } => {
        let Some(a) = out.type_of(store) else {
          todo!();
        };

        let a: Box<dyn Resolve<C>> = Box::new(a);
        let a = std::iter::once(a);
        let a = Box::new(a);

        a
      },
      lang::expr::Expression::Variable { reference, span } => todo!(),
      lang::expr::Expression::Unary { expr, op, span, out } => todo!(),
      lang::expr::Expression::Binary { a, b, op, span, out } => todo!(),
      lang::expr::Expression::StructInitializer { ty, members, span } => todo!(),
    }
  }
}
