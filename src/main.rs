mod lang;
mod tokenize;
mod aster;
mod resolve;
mod generate;

mod lazy;

mod debug;
mod error;
mod settings;

#[cfg(test)] mod test;

use std::process::ExitCode;

use string_pool::StringPool;

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
  let mut lazy = lazy::Lazy::new(&pool, settings);

  match error_handler(&mut lazy, verb) {
    Ok(exit_code) => exit_code,
    Err(message) => {
      error::print_message(&lazy, message);
      ExitCode::FAILURE
    },
  }
}

fn error_handler(
  lazy: &mut lazy::Lazy,
  verb: settings::Verb,
) -> Result<ExitCode, error::PrintableMessage> {
  match verb {
    settings::Verb::Check => {
      lazy.check()?;

      Ok(ExitCode::SUCCESS)
    },
    settings::Verb::Build => {
      lazy.build()?;

      Ok(ExitCode::SUCCESS)
    },
    settings::Verb::Run => {
      lazy.run()
    },
  }
}
