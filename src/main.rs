// For logging macros
#[macro_use] extern crate log;

mod logger;

mod arg_parser;
mod help;

mod compiler;

mod todo;
mod pipeline;

use std::process::ExitCode;

use arg_parser::{
  CompilerOptions,
  error::*,
};

use compiler::{
  Compiler,
  CompilerSettings,
  workflow::DefaultWorkflow,
  error::CompilerError,
};

use crate::help::LazyHelp;
pub(crate) use pipeline::*;

pub(crate) type Result<T = ()> = std::result::Result<T, CompilerError>;

#[allow(non_upper_case_globals)]
pub(crate) const ok: Result = Ok(());

/// Processes the parsed command-line arguments
fn parse_compiler_settings() -> Result<CompilerSettings> {
  let CompilerOptions {
    help,
    input_file,
    output_file,
    llc,
    cc,
  } = arg_parser::parse()?;

  if help {
    return HelpSnafu.fail()?;
  };

  let Some(input_file) = input_file else {
    return NoInputSnafu.fail()?;
  };

  Ok(CompilerSettings {
    input_file,
    output_file,
    llc,
    cc,
  })
}

/// Catch errors in a block so we can deal with them in one place in the main
/// function
fn error_harness() -> Result {
  logger::init();

  let settings = parse_compiler_settings()?;
  let mut compiler = Compiler::<DefaultWorkflow>::new(settings);

  compiler.compile()?;

  ok
}

fn main() -> ExitCode {
  // If an error occurs at any point in the compilation, it bubbles up here
  let Err(err) = error_harness() else {
    // ... if there was none, just exit now
    return ExitCode::SUCCESS;
  };

  // Decide how to present the error to the user; e.g.:
  // the help flag should print the help text
  let should_print_help_text = err.should_print_help_text();
  let should_print_message = err.should_print_message();

  if should_print_help_text {
    help::print_help_text();

    // Put a space between the help text and the error message for clarity
    if should_print_message {
      eprintln!();
    };
  };

  if should_print_message {
    help::print_message(err);
  };

  // Since we have an error, return an error code so the caller is aware
  ExitCode::FAILURE
}
