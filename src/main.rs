#[macro_use] extern crate log;

mod arg_parser;
mod logger;
mod compiler;
mod todo;

mod tokenizer;
mod workflow;

use std::process::exit;

use arg_parser::CompilerOptions;
use compiler::{
  CompilerSettings,
  Compiler,
};
use workflow::DefaultWorkflow;

const HELP_TEXT: fn(executable: &str) -> String = |executable| format!("\
  Usage: {executable} [OPTION]... [INPUT]\n\
  \n\
  Options:\n  \
    -h, --help                             Shows this help message\n  \
    -i, --input=<FILE>                     Sets the program's entry file\n  \
    -o, --output=<FILE>                    Sets the program's output file\n  \
  \n\
  Tooling:\n  \
    --llc=<FILE>                           Path to the llc executable\n  \
    --cc=<FILE>                            Path to the cc executable\n  \
  \n\
  See LICENSE for more information.\
");

fn parse_compiler_settings() -> Result<CompilerSettings, String> {
  let CompilerOptions { help, input_file, output_file, llc, cc } = arg_parser::parse()?;

  if help {
    let full_executable_path = std::env::current_exe().unwrap();
    let executable = full_executable_path.file_name().unwrap().to_string_lossy();

    return Err(HELP_TEXT(&executable));
  };

  let Some(input_file) = input_file else {
    return Err("no input file provided".into());
  };

  Ok(CompilerSettings {
    input_file,
    output_file,
    llc,
    cc,
  })
}

fn error_harness() -> Result<(), String> {
  logger::init();

  let settings = parse_compiler_settings()?;
  let mut compiler = Compiler::<DefaultWorkflow>::new(settings);

  compiler.compile()?;

  todo!()
}

fn main() {
  let result = error_harness();

  if let Err(error) = &result {
    error!("{error}");
  };

  exit(result.is_err() as i32)
}
