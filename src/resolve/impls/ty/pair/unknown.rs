use super::*;

use crate::lang::ty::{Qualified, QualifiedSearchSpace};
use crate::lang::reference::{AliasReference, ModuleReference};
use crate::print_once_per_thread;

pub(super) fn resolve_qualified_to_space(
  lazy: &Lazy,
  module: ModuleReference,
  qualified: &Qualified,
  tasks: &mut Tasks,
) -> Result<Option<QualifiedSearchSpace>> {
  let mut space = qualified.implicit.to_owned();

  for (index, part) in qualified.parts.iter().enumerate() {
    if index == 0 {
      let part_string = lazy.pool.get(part.id);
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
        let borrow = module.rget_from(lazy);

        if let Some(qualified) = borrow.imports.get(&part.id) {
          let Some(new_space) = resolve_qualified_to_space(lazy, module, qualified, tasks)? else {
            print_once_per_thread!(lazy, {
              level: Stub,
              force: false,
              description: line_dbg!("disregarding failed resolution of qualified").into(),
              contents: MessageContents::None,
            });

            return Ok(None);
          };

          space = new_space;

          continue;
        };

        // Look for type aliases by this name
        if let Some(id) = borrow
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

    return tasks.seed_error(ErrorBase::UnknownTypeName {
      module_name: lazy.describe_module(module),
      span: qualified.span,
    });
  };

  Ok(Some(space))
}

pub(super) fn resolve_qualified_to_type(
  lazy: &Lazy,
  module: ModuleReference,
  qualified: &Qualified,
  tasks: &mut Tasks,
) -> Result<Option<Type>> {
  let Some(space) = resolve_qualified_to_space(lazy, module, qualified, tasks)? else {
    return Ok(None);
  };

  Ok(match space {
    QualifiedSearchSpace::Type(ty) => ty.type_of(lazy),
    QualifiedSearchSpace::Intrinsic { kind, span } => Some(Type::Intrinsic { kind, span }),
    QualifiedSearchSpace::Implicit => {
      // not enough info ... do nothing and pray the problem goes away by itself
      None
    },
    other => todo!("{other:#?}"),
  })
}
