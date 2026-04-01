use crate::print_message;

use crate::aster::pprint::Pretty;
use crate::lang::reference::{ModuleReference, Reference};
use crate::lang::Lazy;
use crate::generate::ProgramCompilation;

const DEBUG_PAD_LEN: usize = "debug ".len();

fn indent(source: &str, padding: usize) -> String {
  let spaces = " ".repeat(padding);

  source.split('\n')
    .map(|line| format!("{spaces}{line}"))
    .collect::<Vec<_>>()
    .join("\n")
}

/// Debug source
pub(super) fn source(lazy: &Lazy, global: &ModuleReference) {
  // SPONGE: i'm being lazy here
  let source = global.rget_from(lazy).print(lazy)
    .collect::<Vec<String>>()
    .join("\n");

  let description = indent(&source, DEBUG_PAD_LEN)
    .trim_start()
    .to_owned();

  print_message!(lazy, {
    level: Level::Debug,
    force: false,
    description,
    contents: MessageContents::None,
  });
}

pub(super) fn string_pool(lazy: &Lazy) {
  print_message!(lazy, {
    level: Level::Debug,
    force: false,
    description: format!("{:?}", lazy.pool),
    contents: MessageContents::None,
  });
}

pub(super) fn llvm_source(lazy: &Lazy, compilation: &ProgramCompilation) {
  let source = compilation.dump();
  let description = indent(&source, DEBUG_PAD_LEN + 2)
    .trim_start()
    .to_owned();

  print_message!(lazy, {
    level: Level::Debug,
    force: false,
    description,
    contents: MessageContents::None,
  });
}
