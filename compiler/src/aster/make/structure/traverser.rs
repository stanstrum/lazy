use crate::{print_message, print_once_per_thread};

use crate::resolve::impls::ty::resolve_qualified_to_space;

use super::*;

pub(super) fn traverse_import(
  lazy: &mut lazy::Lazy,
  module: lang::reference::ModuleReference,
  import: &lang::module::import::Import,
) -> Result<(), Error> {
  let mut stack = vec![];

  let count = traverse_group(lazy, &module, &import.source, &import.group, &mut stack)?;

  print_message!(lazy, {
    level: Debug,
    force: false,
    description: format!(line_dbg!("parsed {} import entry(s)"), count),
    contents: MessageContents::WithinSource(WithinSource::new(
      vec![MessageSection {
        text: "here".into(),
        span: import.group.span,
      }],
    )),
  });

  Ok(())
}

fn traverse_group(
  lazy: &mut lazy::Lazy,
  module: &lang::reference::ModuleReference,
  source: &lang::reference::ModuleReference,
  group: &lang::module::import::ImportGroup,
  stack: &mut Vec<lang::module::Name>,
) -> Result<usize, Error> {
  let mut count = 0;

  for part in group.selectors.iter() {
    count += traverse_part(lazy, module, source, part, stack)?;
  };

  Ok(count)
}

fn insert_to_import_map(
  lazy: &mut lazy::Lazy,
  module: &lang::reference::ModuleReference,
  key: string_pool::PoolId,
  value: lang::ty::Qualified,
) -> Result<(), Error> {
  let map = &mut module.rget_from_mut(lazy).transports.import_map;

  if map.contains_key(&key) {
    let id_text = lazy.pool.get(key);

    panic!("id {key:?} already exists: {id_text}");
  };

  map.insert(key, value);

  Ok(())
}

/// Processes a [`lang::module::import::ImportPart`].  Some notes on the params:
/// * `module`: The module parsed the [`lang::module::import::Import`] in the
///   first place.
/// * `source`: The module pointed to by [`lang::module::import::Import::source`]
/// * `part`  : The [`lang::module::import::ImportPart`] to process.
/// * `stack` : The history of [`lang::module::import::ImportQualify`]s that got
///   us to the current point.  This is scope that this should be
///   contexutualized/represented with an [`lang::ty::Qualified`].
fn traverse_part(
  lazy: &mut lazy::Lazy,
  module: &lang::reference::ModuleReference,
  source: &lang::reference::ModuleReference,
  part: &lang::module::import::ImportPart,
  stack: &mut Vec<lang::module::Name>,
)  -> Result<usize, Error> {
  match part {
    &lang::module::import::ImportPart::Star(span) => {
      print_once_per_thread!(lazy, {
        level: Stub,
        force: false,
        description: line_dbg!("restrict ImportPart::Star selector to expored members only").into(),
        contents: MessageContents::File(*module),
      });

      let where_are_we_now = lang::ty::Qualified {
        implicit: lang::ty::QualifiedSearchSpace::Module(*source),
        parts: stack.to_owned(),
        span,
      };

      let space_search = resolve_qualified_to_space(lazy, *source, &where_are_we_now, &None)
        // shouldn't actually throw an error if we don't pass it `tasks`, rather
        // return `None`
        .unwrap();

      let Some(space) = space_search else {
        todo!("error for bad import selector(s): not found");
      };

      module.rget_from_mut(lazy).transports.import_stars.push((space, span));

      Ok(1)
    },
    lang::module::import::ImportPart::Group(group) => traverse_group(lazy, module, source, group, stack),
    lang::module::import::ImportPart::Qualify(qualify) => {
      stack.push(qualify.name);

      let result = match &qualify.next {
        Some(next_part) => traverse_part(lazy, module, source, next_part, stack),
        None => {
          let key = qualify.name.id;
          let value = lang::ty::Qualified {
            implicit: lang::ty::QualifiedSearchSpace::Module(*source),
            parts: stack.to_owned(),
            span: qualify.name.span,
          };

          insert_to_import_map(lazy, module, key, value)?;

          Ok(1)
        },
      };

      stack.pop();

      result
    },
  }
}
