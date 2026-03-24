use crate::lang::reference::{FunctionReference, Reference};
use crate::error::{Level, MessageContents, MessageSection, PrintableMessage, print_message};

use super::*;

fn find_main(lazy: &Lazy, module: ModuleReference) -> Result<FunctionReference> {
  let main_search = {
    let main_id = lazy.pool.insert("main");

    module.rget_from(lazy)
    .functions.iter()
    .find(|&function| {
      function.rget_from(lazy)
        .header.name.id == main_id
    })
  };

  let Some(main) = main_search else {
    let root = lazy.get_root_module(module);
    let module_name = lazy.describe_module(root);

    return Err(Box::new(Error::MissingEntryPoint {
      module_name,
      file: module,
    }));
  };

  {
    let module_name = lazy.describe_module(module);
    let function = main.rget_from(lazy);
    let span = function.header.name.span;

    print_message(lazy, PrintableMessage {
      level: Level::Debug,
      force: false,
      description: format!("{module_name} has the entrypoint \"main\""),
      contents: MessageContents::WithinSource {
        range: span,
        sections: vec![MessageSection {
          text: "here".into(),
          span,
        }],
      },
    });
  };

  Ok(*main)
}

pub(super) fn program(lazy: &Lazy, module: ModuleReference) -> Result<()> {
  let main = find_main(lazy, module)?;

  todo!("verify program")
}
