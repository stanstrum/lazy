// For logging macros
#[macro_use]
extern crate log;

mod logger;
mod help;

mod arg_parser;
mod pipeline;
mod compiler;

#[cfg(test)] mod test;

use std::process::ExitCode;

use arg_parser::error::*;
pub(crate) use pipeline::*;
use compiler::{Compiler, workflow::DefaultWorkflow, error::CompilerError};

pub(crate) type Result<T = ()> = std::result::Result<T, CompilerError>;

#[allow(non_upper_case_globals)]
pub(crate) const ok: Result = Ok(());

fn main() -> ExitCode {
  logger::init();

  // The first argument is typically the executable path -- ignore that
  let args = std::env::args().skip(1);

  // Catch errors in a block so we can deal with them in one place
  let error_harness = || {
    let settings = arg_parser::parse(args)?;
    let mut compiler = Compiler::<DefaultWorkflow>::new(settings)?;

    compiler.compile()
  };

  // If an error occurs at any point in the compilation, it bubbles up here
  let Err(err) = error_harness() else {
    // ... if there was none, just exit now
    return ExitCode::SUCCESS;
  };

  // Write the error to the logger
  err.output_to_logger();

  // Since we have an error, return an error code so the caller is aware
  ExitCode::FAILURE
}
