mod string_pool;

mod lang;
mod tokenize;
mod aster;
mod resolve;
mod generate;

mod error;
mod settings;

#[cfg(test)] mod test;

use std::process::ExitCode;

use lang::Lazy;

use crate::lang::reference::{ModuleReference, Store};
use crate::string_pool::StringPool;

use crate::aster::pprint::Pretty;

fn main() -> ExitCode {
  let args = std::env::args();
  run_with(args.into_iter())
}

fn error_handler<'lazy, 'pool>(lazy: &'lazy mut Lazy<'pool>, global: ModuleReference, verb: settings::Verb) -> Result<(), error::PrintableMessage> {
  aster::asterize(lazy, global)?;
  resolve::resolve_and_verify(lazy, global)?;

  let source = lazy.rget(global).print(&lazy)
    .map(|s| format!(line_dbg!("{}"), s))
    .collect::<Vec<_>>()
    .join("\n");

  println!("{source}");
  println!("{:?}", &lazy.pool);

  if matches!(verb, settings::Verb::Check) {
    return Ok(());
  };

  let args = generate::args::CliArgs {
    target: None,
    opt_level: generate::args::OptimizationLevel::O0,
    passes: "instcombine,reassociate,gvn,simplifycfg,mem2reg".into(),
  };
  // file_type: inkwell::targets::FileType::Object,
  // out_path: lazy.settings.output_path.clone(),

  let program = generate::Program::new(global, args);
  let compilation = program.compile(lazy)?;

  compilation.debug();
  // compilation.optimize();

  // TODO: get this from settings
  let file_type = inkwell::targets::FileType::Object;
  let _result = compilation.save_to_file(file_type, &lazy.settings.output_path)?;

  if matches!(verb, settings::Verb::Build) {
    return Ok(());
  };

  todo!("run executable");

  Ok(())
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
    Ok(()) => ExitCode::SUCCESS,
    Err(message) => {
      error::print_message(&lazy, message);
      ExitCode::FAILURE
    },
  }
}
