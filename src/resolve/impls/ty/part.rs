use crate::lang::reference::TypePartReference;
use crate::resolve::SpecialPair;

use super::*;

impl<'a, 'b> TypePair<'a, 'b> {
  pub fn new(reference: &'a TypeReference, ty: &'a Type) -> Self {
    Self {
      pair: SpecialPair(reference, ty),
      modifiers: vec![],
    }
  }
}

impl<'a, 'b> TypeOf for TypePair<'a, 'b> {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    self.pair.type_of(lazy)
  }

  fn reference(&self, lazy: &Lazy) -> Option<TypeReference> {
    self.pair.reference(lazy)
  }
}


impl Resolve for TypePartReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    // let description = format!(line_dbg!("Resolve TypePartReference: {}"), self.print(lazy));

    // tasks.work(description, |tasks| {
    let reference = TypeReference::Part(*self);
    let ty = self.rget_from(lazy);

    TypePair::new(&reference, ty).resolve(lazy, tasks)
    // })
  }
}

impl Coerce for TypePartReference {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let reference = TypeReference::Part(*self);
    let ty = self.rget_from(lazy);

    TypePair::new(&reference, ty).coerce(lazy, other, tasks)
  }
}
