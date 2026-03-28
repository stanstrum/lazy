use super::*;

use crate::lang::ty::{Qualified, QualifiedSearchSpace};
use crate::lang::reference::{AliasReference, ModuleReference};

pub(super) fn resolve_qualified_to_type(lazy: &Lazy, module: ModuleReference, qualified: &Qualified) -> Result<Option<Type>> {
  let mut space = qualified.implicit.to_owned();

  for (count, part) in qualified.parts.iter().enumerate() {
    if count == 0 {
      let part_string = lazy.pool.get(part.id).collect::<String>();
      if let Some(kind) = Intrinsic::try_from_str(&part_string) {
        space = QualifiedSearchSpace::Intrinsic {
          kind,
          span: part.span,
        };

        continue;
      };
    };

    match space {
      // QualifiedSearchSpace::Type(ty) => todo!("match space: {ty:#?}"),
      QualifiedSearchSpace::Module(module) => {
        // Look for type aliases by this name
        if let Some(id) = module.rget_from(lazy)
          .aliases.iter()
          .position(|alias| part.id == alias.name.id)
        {
          let alias = AliasReference(module, id);
          space = QualifiedSearchSpace::Type(TypeReference::Alias(alias).into());

          continue;
        };
      },
      other => todo!("{other:?}"),
    };

    return Err(Box::new(Error::UnknownTypeName {
      module_name: lazy.describe_module(module),
      span: qualified.span,
    }))
  };

  Ok(match space {
    QualifiedSearchSpace::Type(ty) => ty.type_of(lazy)?,
    QualifiedSearchSpace::Intrinsic { kind, span } => Some(Type::Intrinsic { kind, span }),
    QualifiedSearchSpace::Implicit => {
      // not enough info ... do nothing and pray the problem goes away by itself
      None
    },
    other => todo!("{other:#?}"),
  })
}
