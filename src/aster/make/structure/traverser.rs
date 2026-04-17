use crate::print_message;

use super::*;

pub(super) fn traverse_import(
  lazy: &mut lang::Lazy,
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
  lazy: &mut lang::Lazy,
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

fn traverse_part(
  lazy: &mut lang::Lazy,
  module: &lang::reference::ModuleReference,
  source: &lang::reference::ModuleReference,
  part: &lang::module::import::ImportPart,
  stack: &mut Vec<lang::module::Name>,
)  -> Result<usize, Error> {
  match part {
    lang::module::import::ImportPart::Star(_) => todo!(),
    lang::module::import::ImportPart::Group(group) => traverse_group(lazy, module, source, group, stack),
    lang::module::import::ImportPart::Qualify(qualify) => {
      stack.push(qualify.name);

      let result = match &qualify.next {
        Some(next_part) => traverse_part(lazy, module, source, next_part, stack),
        None => {
          let id = qualify.name.id;
          let map = &mut module.rget_from_mut(lazy).imports;

          if map.contains_key(&id) {
            let id_text = lazy.pool.get(id);

            panic!("id {id:?} already exists: {id_text}");
          };

          map.insert(id, lang::ty::Qualified {
            implicit: lang::ty::QualifiedSearchSpace::Module(*source),
            parts: stack.to_owned(),
            span: qualify.name.span,
          });

          Ok(1)
        },
      };

      stack.pop();

      result
    },
  }
}
