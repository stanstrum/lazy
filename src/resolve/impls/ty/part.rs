use crate::lang::reference::TypePartReference;

use super::*;

impl Resolve for TypePartReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    // let description = format!(line_dbg!("Resolve TypePartReference: {}"), self.print(lazy));

    // tasks.work(description, |tasks| {
    let reference = TypeReference::Part(*self);
    let ty = self.rget_from(lazy);

    SpecialPair(&reference, ty).resolve(lazy, tasks)
    // })
  }
}

impl Coerce for TypePartReference {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let reference = TypeReference::Part(*self);
    let ty = self.rget_from(lazy);

    SpecialPair(&reference, ty).coerce(lazy, other, tasks)
  }
}
