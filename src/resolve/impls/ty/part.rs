use crate::lang::reference::TypePartReference;

use super::*;

impl TypeOf for TypePartReference {
  fn type_of(&self, lazy: &Lazy) -> Option<Type> {
    self.rget_from(lazy).type_of(lazy)
  }

  fn reference(&self, lazy: &Lazy) -> Option<OverwriteTypeReference> {
    Some(TypeReference::Part(*self).into())
  }
}

impl Resolve for TypePartReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    // let description = format!(line_dbg!("Resolve TypePartReference: {}"), self.print(lazy));

    // tasks.work(description, |tasks| {
    TypeReference::Part(*self).resolve(lazy, tasks)
    // })
  }
}

impl Coerce for TypePartReference {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    TypeReference::Part(*self).coerce(lazy, other, tasks)
  }
}
