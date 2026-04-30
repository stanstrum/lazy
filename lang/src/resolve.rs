use crate::{Compiler, CompilerPoolStore, ty::{Qualified, QualifiedSearchSpace}};

#[derive(Debug)]
pub enum QualifiedError {

}

/// This function takes in a [`Qualified`] reference and return the
/// corresponding [`QualifiedSearchSpace`] or [`None`] if none is found.
///
/// Returns a unique [`QualifiedError`] as this method is used by more than one
/// module in the compiler.
///
/// Parameters:
/// - store: The [`crate::reference::Store`] in use
/// - qualified: The [`Qualified`] to resolve
/// - check_first: Whether to treat the first part of this qualified as shorthand
///     for the standard library or internal intrinsic types (see [`crate::intrinsic::Intrinsic`])
pub fn resolve_qualified_to_space<C: Compiler>(store: &C::Store<'_>, qualified: &Qualified<C>, check_first: bool) -> Result<Option<QualifiedSearchSpace<C>>, QualifiedError> {
  let mut space = qualified.implicit;
  let super_id = store.pool_keys().super_;

  // For each part ...
  for (index, part) in qualified.parts.iter().enumerate() {
    let is_first = index == 0;
    let is_other_than_implicit = !matches!(space, QualifiedSearchSpace::Intrinsic { .. });

    // First, check if this is the first part of this qualified and it's not
    // implicit
    if check_first && is_first && is_other_than_implicit {
      if let Some(kind) = crate::intrinsic::Intrinsic::try_from_keys(part.id, store.pool_keys()) {
        space = QualifiedSearchSpace::Intrinsic {
          kind,
          span: part.span,
        };
        continue;
      };

      todo!("check for belonging to @std");
    };

    todo!("check other things")
  };

  Ok(Some(space))
}
