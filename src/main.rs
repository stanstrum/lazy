#[macro_use] extern crate log;

mod help;

mod arg_parser;
mod logger;
mod compiler;
mod todo;

mod tokenizer;
mod workflow;

use std::process::ExitCode;

use arg_parser::{
  CompilerOptions,
  error::*,
};

use compiler::{
  Compiler,
  CompilerSettings,
  CompilerResult,
};

use workflow::DefaultWorkflow;

pub(crate) trait LazyHelp {
  fn should_print_message(&self) -> bool {
    true
  }

  fn should_print_help_text(&self) -> bool {
    false
  }
}

fn parse_compiler_settings() -> CompilerResult<CompilerSettings> {
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

fn error_harness() -> CompilerResult<()> {
  logger::init();

  let settings = parse_compiler_settings()?;
  let mut compiler = Compiler::<DefaultWorkflow>::new(settings);

  compiler.compile()?;

  todo!()
}

fn main() -> ExitCode {
  let Err(err) = error_harness() else {
    return ExitCode::SUCCESS;
  };

  let should_print_help_text = err.should_print_help_text();
  let should_print_message = err.should_print_message();

  if should_print_help_text {
    help::print_help_text();

    if should_print_message {
      eprintln!();
    };
  };

  if should_print_message {
    error!("{err}");
  };

  ExitCode::FAILURE
}
