use super::*;

use crate::lang::ty::Qualified;
use crate::lang::reference::{AliasReference, ModuleReference};

pub(super) fn resolve_qualified_to_type(lazy: &Lazy, module: ModuleReference, qualified: &Qualified) -> Result<Option<Type>> {
  #[derive(Debug)]
  enum QualifiedSearchSpace {
    Type(Type),
    Module(ModuleReference),
  }

  if qualified.implicit {
    // Implicits are coerced before they are resolved
    return Ok(None);
  };

  let mut space = QualifiedSearchSpace::Module(module);

  for (count, part) in qualified.parts.iter().enumerate() {
    if count == 0 {
      let part_string = lazy.pool.get(part.id).collect::<String>();
      if let Some(kind) = Intrinsic::try_from_str(&part_string) {
        space = QualifiedSearchSpace::Type(
          Type::Intrinsic {
            kind,
            span: part.span,
          }
        );

        continue;
      };
    };

    match space {
      QualifiedSearchSpace::Type(ty) => todo!("match space: {ty:#?}"),
      QualifiedSearchSpace::Module(module) => {
        // Look for type aliases by this name
        if let Some(id) = module.rget_from(lazy)
          .aliases.iter()
          .position(|alias| part.id == alias.name.id)
        {
          let alias = AliasReference(module, id);
          space = QualifiedSearchSpace::Type(Type::Reference(TypeReference::Alias(alias)));

          continue;
        };
      },
    };

    return Err(Box::new(Error::UnknownTypeName {
      module_name: lazy.describe_module(module),
      span: qualified.span,
    }))
  };

  match space {
    QualifiedSearchSpace::Type(ty) => Ok(Some(ty)),
    QualifiedSearchSpace::Module(_) => todo!(),
  }
}
