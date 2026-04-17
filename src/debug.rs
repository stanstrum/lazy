use std::process::Command;

use crate::print_message;

use crate::aster::pprint::Pretty;
use crate::lang::reference::{ModuleReference, Reference};
use crate::lang::Lazy;
use crate::generate::{ProgramCompilation, ProgramObjectFile};

const ERROR_PAD_LEN: usize = "error ".len();
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
    level: Debug,
    force: false,
    description,
    contents: MessageContents::None,
  });
}

pub(super) fn string_pool(lazy: &Lazy) {
  print_message!(lazy, {
    level: Debug,
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
    level: Debug,
    force: false,
    description,
    contents: MessageContents::None,
  });
}

pub(super) fn object_file(lazy: &Lazy, object_file: &ProgramObjectFile) {
  let mut builder = std::process::Command::new("stat");

  let command = builder
    .arg(&object_file.path)
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped());

  let command_text = format!("Running `{command:?}`");

  let output = command.spawn()
    .expect("to spawn stat subprocess")
    .wait_with_output()
    .expect("to wait for stat subprocess");

  let stdout = std::str::from_utf8(&output.stdout)
    .expect("to parse stdout");
  let stderr = std::str::from_utf8(&output.stderr)
    .expect("to parse stderr");

  let exit_status = if output.status.success() {
    "stat exited successfully."
  } else if let Some(exit_code) = output.status.code() {
    &format!("stat exited with exit code {exit_code}")
  } else {
    "stat exited unsucessfully."
  };

  if !stderr.is_empty() {
    let description = indent(stderr, ERROR_PAD_LEN);

    print_message!(lazy, {
      level: Error,
      force: false,
      description,
      contents: MessageContents::None,
    });
  };

  let altogether = [
    &command_text,
    stdout,
    exit_status,
  ].join("\n");

  let description = indent(&altogether, DEBUG_PAD_LEN)
    .trim_start()
    .to_owned();

  print_message!(lazy, {
    level: Debug,
    force: false,
    description,
    contents: MessageContents::None,
  });
}

pub(super) fn subprocess_command(lazy: &Lazy, command: &Command) {
  let command_text = format!("{command:?}");
  let description = format!("Running `{command_text}`");

  print_message!(lazy, {
    level: Info,
    force: false,
    description,
    contents: MessageContents::None,
  });
}
