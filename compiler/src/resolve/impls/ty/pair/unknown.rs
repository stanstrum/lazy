use super::*;

use crate::lang::module::ModuleParent;
use lang::CompilerPoolStore;
use lazy_macros::{print_message, print_once_per_thread};

use crate::lang::ty::{Qualified, QualifiedSearchSpace};
use gluezy::ModuleReference;
use ::lang::reference::{AliasReference, StructReference};

pub(crate) fn resolve_qualified_to_space(
  lazy: &Lazy,
  module: ModuleReference,
  qualified: &Qualified,
  tasks: &Option<&mut Tasks<LazyStructures>>,
) -> Result<Option<QualifiedSearchSpace>> {
  let mut space = qualified.implicit.to_owned();

  let seed = |base: ErrorBase| -> Result<Option<QualifiedSearchSpace>> {
    if let Some(tasks) = tasks {
      tasks.seed_error(base)
    } else {
      Ok(None)
    }
  };

  // For each part ...
  'part_match: for part in qualified.parts.iter() {
    // println!(line_dbg!("resolve_qualified_to_space: search for {} in {}"), part.print(lazy), space.print(lazy));

    // First thing, if it's super, we'll do that first.  I don't mind having
    // 'super' be something you can't name something, without it being a Keyword,
    // no yeah, it doesn't bother me at all :(
    if part.id == lazy.pool_keys.super_ {
      print_message!(lazy, {
        level: Debug,
        force: false,
        description: format!(line_dbg!("super keyword in {}"), lazy.describe_module(part.span.module)),
        contents: MessageContents::WithinSource(WithinSource::new(
          vec![MessageSection {
            text: format!("space is {}", space.print(lazy)),
            span: part.span,
          }],
        )),
      });

      let QualifiedSearchSpace::Module(space_module) = space else {
        return seed(ErrorBase::BadQualify {
          span: part.span,
        });
      };

      let ModuleParent::Module(parent) = lazy.rget(space_module).parent else {
        return seed(ErrorBase::BadQualify {
          span: part.span,
        });
      };

      space = QualifiedSearchSpace::Module(parent);
      continue;
    };

    if // Special behaviors for first, non-implicit part
      !matches!(&qualified.implicit, QualifiedSearchSpace::Implicit)
    {
      // If the first part is an intrinsic, make it so
      let part_string = lazy.pool.get(part.id);
      if let Some(kind) = Intrinsic::try_from_str(&part_string) {
        space = QualifiedSearchSpace::Intrinsic {
          kind,
          span: part.span,
        };
        continue;
      };

      // If the first part is something that can be found in `std`, do that.
      // We must make sure we aren't stuck in an infinite loop so that the
      // standard library doesn't try to go look itself up recursively
      if let QualifiedSearchSpace::Module(space_module) = space {
        let std = lazy.std.unwrap();

        if space_module != std && module != std {
          let std_qualified = Qualified {
            implicit: QualifiedSearchSpace::Module(std),
            parts: vec![*part],
            span: part.span,
          };

          // Search std for the part
          if let Ok(Some(next_space)) = resolve_qualified_to_space(lazy, module, &std_qualified, tasks) {
            space = next_space;
            continue;
          };
        };
      };
    };

    match space {
      // QualifiedSearchSpace::Type(ty) => todo!("match space: {ty:#?}"),
      QualifiedSearchSpace::Module(current_module) => {
        let borrow = current_module.rget_from(lazy);

        if let Some(qualified) = borrow.transports.import_map.get(&part.id) {
          let Some(new_space) = resolve_qualified_to_space(lazy, current_module, qualified, tasks)? else {
            print_once_per_thread!(lazy, {
              level: Stub,
              force: false,
              description: line_dbg!("disregarding failed resolution of qualified").into(),
              contents: MessageContents::None::<LazyStructures>,
            });

            return Ok(None);
          };

          // replace the space and search from there
          space = new_space;
          continue 'part_match;
        };

        for (wildcard_space, span) in borrow.transports.import_stars.iter() {
          if
            let QualifiedSearchSpace::Module(wildscare_space_module) = wildcard_space &&
            let QualifiedSearchSpace::Module(space_module) = &space &&
            wildscare_space_module == space_module
          {
            let module_name = lazy.describe_module(*wildscare_space_module);
            let description = format!(line_dbg!("BUGBGUG: Module {} contains itself as an import star selector ... this will cause crashes."), module_name);

            print_message!(lazy, {
              level: Warn,
              force: false,
              description,
              contents: MessageContents::File::<LazyStructures>(*wildscare_space_module),
            });

            continue;
          };

          let test_qualified = Qualified {
            implicit: wildcard_space.to_owned(),
            parts: vec![*part],
            span: *span,
          };

          if let Ok(Some(next_space)) = resolve_qualified_to_space(lazy, current_module, &test_qualified, tasks) {
            // replace the space and search from there
            space = next_space;
            continue 'part_match;
          };
        };

        // Look for submodules by this name
        if let Some(found_submodule) = borrow
          .modules.iter()
          .find(|submodule| part.id == (*submodule).rget_from(lazy).name)
        {
          space = QualifiedSearchSpace::Module(*found_submodule);
          continue 'part_match;
        };

        // Look for type aliases by this name
        if let Some(id) = borrow
          .aliases.iter()
          .position(|alias| part.id == alias.name.id)
        {
          let alias = AliasReference(current_module, id);

          // replace the space and search from there
          space = QualifiedSearchSpace::Type(TypeReference::Alias(alias).into());
          continue 'part_match;
        };

        // Look for structs by this name
        if let Some(id) = borrow
          .structs.iter()
          .position(|struc| struc.name.id == part.id)
        {
          let struct_reference = StructReference(current_module, id);

          // replace the yadda yadda
          space = QualifiedSearchSpace::Struct(struct_reference);
          continue 'part_match;
        };
      },
      other => todo!("{other:?}"),
    };

    // fallthrough if no matches; yield error
    return seed(ErrorBase::UnknownTypeName {
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
  tasks: &mut Tasks<LazyStructures>,
) -> Result<Option<Type>> {
  Ok(match resolve_qualified_to_space(lazy, module, qualified, &Some(tasks))? {
    Some(QualifiedSearchSpace::Type(ty)) => ty.type_of(lazy),
    Some(QualifiedSearchSpace::Intrinsic { kind, span }) => Some(Type::Intrinsic { kind, span }),
    Some(QualifiedSearchSpace::Struct(prototype)) => Some(Type::Struct { prototype }),
    Some(QualifiedSearchSpace::Implicit) => {
      // not enough info ... do nothing and pray the problem goes away by itself
      None
    },
    None => None,
    other => todo!("{other:#?}"),
  })
}
