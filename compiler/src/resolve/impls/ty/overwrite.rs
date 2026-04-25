use super::*;

impl Coerce for OverwriteTypeReference {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()> {
    let Some(ty) = self.type_of(lazy) else {
      return Ok(());
    };

    TypePair {
      overwrite: self.clone(),
      ty,
    }.coerce(lazy, other, tasks)
  }
}
