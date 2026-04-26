#[cfg(test)] mod test;

pub use string_pool::StringPool;

mod debug;

use std::path::Path;
use std::process::ExitCode;
use std::os::unix::fs::PermissionsExt;

use lang::CompilerPoolStore;
use lazy_macros::print_message;
use gluezy::{Lazy, LazyStructures, ModuleReference};

pub fn check(lazy: &mut Lazy<LazyStructures>) -> Result<ModuleReference, log::PrintableMessage<LazyStructures>> {
  // Instantiate the global scope
  let path = lazy.settings.input_path.to_owned();
  let global = lazy.add_file("@global", path, None)?;

  // Resolve, verify
  resolve::resolve_and_verify(lazy, global)?;

  // Debugs
  debug::source(lazy, &global);
  debug::string_pool(lazy);

  Ok(global)
}

pub fn build<'a>(lazy: &'a mut gluezy::Lazy) -> Result<&'a Path, log::PrintableMessage<LazyStructures>> {
  let global = check(lazy)?;

  // Otherwise, let's go build the module
  let args = generate::args::CliArgs {
    target: None,
    opt_level: generate::args::OptimizationLevel::O0,
    passes: "instcombine,reassociate,gvn,simplifycfg,mem2reg".into(),
  };

  let program = generate::Program::new(global, args);
  let compilation = program.compile(lazy)?;

  // Debug the LLVM source
  debug::llvm_source(lazy, &compilation);

  // Optimize the IR
  compilation.optimize(lazy)?;
  debug::llvm_source(lazy, &compilation);

  // Write out the object file for the global module
  // TODO: get this from settings
  let file_type = generate::FileType::Object;
  let object_file = compilation.save_to_file(file_type)?;

  debug::object_file(lazy, &object_file);

  // TODO: find out what needs to be linked
  let linked = [
    "c", // libc
  ];

  // Link the module and write the program to the output file
  let executable = object_file.link_with(&lazy.settings.output_path, &linked)?;

  // Set the output file's permissions to be executable
  let perms = std::fs::Permissions::from_mode(0o755);
  std::fs::set_permissions(executable, perms)
    .expect("failed to chmod 755 {executable:?}");

  Ok(executable)
}

pub fn run(lazy: &mut gluezy::Lazy) -> Result<ExitCode, log::PrintableMessage<LazyStructures>> {
  let executable = build(lazy)?;

  // Otherwise, go run the child program
  let mut command = std::process::Command::new(executable);
  debug::subprocess_command(lazy, &command);

  let mut child = command
    // .args(args);
    .spawn()
    .expect("to launch {executable:?}");

  // Wait on the child and get an exit status
  let exit_status = child.wait()
    .expect("to wait on child process");

  // Print that info and set our own exit code accordingly
  let (exit_code, level, message) = if exit_status.success() {
    (
      ExitCode::SUCCESS, log::Level::Info,
      "Program exited successfully.".into(),
    )
  } else if let Some(code) = exit_status.code() {
    (
      ExitCode::FAILURE, log::Level::Error,
      format!("Program exited with status code {code}."),
    )
  } else {
    (
      ExitCode::FAILURE, log::Level::Error,
      "Program exited unsuccessfully.".into(),
    )
  };

  print_message!(lazy, {
    level,
    force: false,
    description: message,
    contents: MessageContents::None::<LazyStructures>,
  });

  Ok(exit_code)
}
