mod string_pool;

mod lang;
mod tokenize;
mod aster;
mod resolve;
mod generate;

mod debug;
mod error;
mod settings;

#[cfg(test)] mod test;

use std::os::unix::fs::PermissionsExt;
use std::process::ExitCode;

use lang::Lazy;

use crate::lang::reference::{ModuleReference, Store};
use crate::string_pool::StringPool;

use crate::aster::pprint::Pretty;

fn main() -> ExitCode {
  let args = std::env::args();
  run_with(args.into_iter())
}

fn run_with(args: impl Iterator<Item = String>) -> ExitCode {
  let (settings, verb) = match settings::parse_and_display(args) {
    Ok(settings) => settings,
    Err(exit_code) => return exit_code,
  };

  let pool = StringPool::new();
  let mut lazy = Lazy::new(&pool, settings);

  let path = lazy.settings.input_path.to_owned();
  let global = lazy.add_file("global", path);

  match error_handler(&mut lazy, global, verb) {
    Ok(exit_code) => exit_code,
    Err(message) => {
      error::print_message(&lazy, message);
      ExitCode::FAILURE
    },
  }
}

fn error_handler<'lazy, 'pool>(
  lazy: &'lazy mut Lazy<'pool>,
  global: ModuleReference,
  verb: settings::Verb,
) -> Result<ExitCode, error::PrintableMessage> {
  // Tokenize, asterize (parse AST)
  aster::asterize(lazy, global)?;

  // Resolve, verify
  resolve::resolve_and_verify(lazy, global)?;

  // Debugs
  debug::source(lazy, &global);
  debug::string_pool(lazy);

  // Stop here if all we wanted was to check
  if matches!(verb, settings::Verb::Check) {
    return Ok(ExitCode::SUCCESS);
  };

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

  // // Optimize the IR
  // compilation.optimize();

  // Write out the object file for the global module
  // TODO: get this from settings
  let file_type = inkwell::targets::FileType::Object;
  let object_file = compilation.save_to_file(file_type)?;

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

  // Stop here if all we wanted was to build
  if matches!(verb, settings::Verb::Build) {
    return Ok(ExitCode::SUCCESS);
  };

  // Otherwise, go run the child program
  let mut command = std::process::Command::new(executable)
    // .args(args);
    .spawn()
    .expect("to launch {executable:?}");

  // Wait on the child and get an exit status
  let exit_status = command.wait()
    .expect("to wait on child process");

  // Print that info and set our own exit code accordingly
  let exit_code = if exit_status.success() {
    println!("Program exited successfully.");
    ExitCode::SUCCESS
  } else if let Some(code) = exit_status.code() {
    println!("Program exited with status code {code}.");
    ExitCode::FAILURE
  } else {
    println!("Program exited unsuccessfully.");
    ExitCode::FAILURE
  };

  Ok(exit_code)
}
