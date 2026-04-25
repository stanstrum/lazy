use gluezy::TypePartReference;

use super::*;

impl TypeOf<LazyStructures> for TypePartReference {
  fn type_of(&self, lazy: &Lazy) -> Option<Type> {
    self.rget_from(lazy).type_of(lazy)
  }

  fn reference(&self, _lazy: &Lazy) -> Option<OverwriteTypeReference> {
    Some(TypeReference::Part(*self).into())
  }
}

impl Resolve for TypePartReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks<LazyStructures>) -> Result<()> {
    // let description = format!(line_dbg!("Resolve TypePartReference: {}"), self.print(lazy));

    // tasks.work(description, |tasks| {
    TypeReference::Part(*self).resolve(lazy, tasks)
    // })
  }
}

impl Coerce for TypePartReference {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf<LazyStructures>, tasks: &mut Tasks<LazyStructures>) -> Result<()> {
    TypeReference::Part(*self).coerce(lazy, other, tasks)
  }
}
