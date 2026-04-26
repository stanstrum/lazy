use lang::reference::TypePartReference;

use super::*;

impl<C: Compiler + 'static> Resolve<C> for TypePartReference<C> {
  fn resolve(&self, store: &C::Store<'_>, tasks: &mut Tasks<C>) -> Result<C> {
    // let description = format!(line_dbg!("Resolve TypePartReference: {}"), self.print(lazy));

    // tasks.work(description, |tasks| {
    TypeReference::Part(*self).resolve(store, tasks)
    // })
  }
}

impl<C: Compiler + 'static> Coerce<C> for TypePartReference<C> {
  fn coerce(&self, lazy: &C::Store<'_>, other: &impl TypeOf<C>, tasks: &mut Tasks<C>) -> Result<C> {
    TypeReference::Part(*self).coerce(lazy, other, tasks)
  }
}
