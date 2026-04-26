use super::*;

use lang::module::ModuleParent;
use lang::CompilerPoolStore;
use lazy_macros::{print_message, print_once_per_thread};

use lang::ty::{Qualified, QualifiedSearchSpace};
use lang::reference::{AliasReference, StructReference};

pub fn resolve_qualified_to_space<C: Compiler + 'static>(
  store: &C::Store<'_>,
  module: C::ModuleReference,
  qualified: &Qualified<C>,
  tasks: &Option<&mut Tasks<C>>,
) -> Result<C, Option<QualifiedSearchSpace<C>>> {
  let mut space = qualified.implicit.to_owned();

  let seed = |base: ResolveErrorBase<C>| -> Result<C, Option<QualifiedSearchSpace<C>>> {
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
    if part.id == store.pool_keys().super_ {
      print_message!(store, {
        level: Debug,
        force: false,
        description: format!(line_dbg!("super keyword in {}"), store.describe_module(part.span.module)),
        contents: MessageContents::WithinSource(WithinSource::new(
          vec![MessageSection {
            text: format!("space is {}", space.print(store)),
            span: part.span,
          }],
        )),
      });

      let QualifiedSearchSpace::Module(space_module) = space else {
        return seed(ResolveErrorBase::BadQualify {
          span: part.span,
        });
      };

      let ModuleParent::Module(parent) = store.rget(space_module).parent else {
        return seed(ResolveErrorBase::BadQualify {
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
      let part_string = store.pool().get(part.id);
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
        let std = store.unwrap_std();

        if space_module != std && module != std {
          let std_qualified = Qualified {
            implicit: QualifiedSearchSpace::Module(std),
            parts: vec![*part],
            span: part.span,
          };

          // Search std for the part
          if let Ok(Some(next_space)) = resolve_qualified_to_space(store, module, &std_qualified, tasks) {
            space = next_space;
            continue;
          };
        };
      };
    };

    match space {
      // QualifiedSearchSpace::Type(ty) => todo!("match space: {ty:#?}"),
      QualifiedSearchSpace::Module(current_module) => {
        let borrow = current_module.rget_from(store);

        if let Some(qualified) = borrow.transports.import_map.get(&part.id) {
          let Some(new_space) = resolve_qualified_to_space(store, current_module, qualified, tasks)? else {
            print_once_per_thread!(store, {
              level: Stub,
              force: false,
              description: line_dbg!("disregarding failed resolution of qualified").into(),
              contents: MessageContents::None::<C>,
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
            let module_name = store.describe_module(*wildscare_space_module);
            let description = format!(line_dbg!("BUGBGUG: Module {} contains itself as an import star selector ... this will cause crashes."), module_name);

            print_message!(store, {
              level: Warn,
              force: false,
              description,
              contents: MessageContents::File::<C>(*wildscare_space_module),
            });

            continue;
          };

          let test_qualified = Qualified {
            implicit: wildcard_space.to_owned(),
            parts: vec![*part],
            span: *span,
          };

          if let Ok(Some(next_space)) = resolve_qualified_to_space(store, current_module, &test_qualified, tasks) {
            // replace the space and search from there
            space = next_space;
            continue 'part_match;
          };
        };

        // Look for submodules by this name
        if let Some(found_submodule) = borrow
          .modules.iter()
          .find(|submodule| part.id == (*submodule).rget_from(store).name)
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
    return seed(ResolveErrorBase::UnknownTypeName {
      module_name: store.describe_module(module),
      span: qualified.span,
    });
  };

  Ok(Some(space))
}

pub(super) fn resolve_qualified_to_type<C: Compiler + 'static>(
  store: &C::Store<'_>,
  module: C::ModuleReference,
  qualified: &Qualified<C>,
  tasks: &mut Tasks<C>,
) -> Result<C, Option<Type<C>>> {
  Ok(match resolve_qualified_to_space(store, module, qualified, &Some(tasks))? {
    Some(QualifiedSearchSpace::Type(ty)) => ty.type_of(store),
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
