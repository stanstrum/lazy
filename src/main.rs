mod string_pool;

mod lang;
mod tokenize;
mod aster;
mod resolve;
mod generate;

mod error;
mod settings;

use std::process::ExitCode;

use lang::Lazy;

use crate::lang::reference::Store;
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

  let error_handler: Result<(), error::PrintableMessage> = 'error: {
    if let Err(err) = aster::asterize(&mut lazy, &pool, global) {
      break 'error Err(err.into());
    };

    if let Err(err) = resolve::resolve_and_verify(&mut lazy, global) {
      break 'error Err((*err).into());
    };

    let source = lazy.rget(global).print(&lazy)
        .map(|s| format!(line_dbg!("{}"), s))
        .collect::<Vec<_>>()
        .join("\n");

      println!("{source}");


    println!("{:?}", &lazy.pool);

    if matches!(verb, settings::Verb::Check) {
      break 'error Ok(());
    };

    if let Err(err) = generate::compile(&lazy, global) {
      break 'error Err(err.into());
    };

    if matches!(verb, settings::Verb::Build) {
      break 'error Ok(());
    };

    todo!("run");

    Ok(())
  };

  match error_handler {
    Ok(()) => ExitCode::SUCCESS,
    Err(message) => {
      error::print_message(&lazy, message);
      ExitCode::FAILURE
    },
  }
}

#[cfg(test)]
mod test;
