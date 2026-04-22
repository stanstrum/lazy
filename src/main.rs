mod info;
mod parse;

use std::process::ExitCode;

fn main() -> ExitCode {
  let args = std::env::args();
  run_with(args.into_iter())
}

fn run_with(args: impl Iterator<Item = String>) -> ExitCode {
  let (settings, verb) = match parse::parse_and_display(args) {
    Ok(settings) => settings,
    Err(exit_code) => return exit_code,
  };

  let pool = compiler::StringPool::new();
  let mut lazy = compiler::lazy::Lazy::new(&pool, settings);

  match error_handler(&mut lazy, verb) {
    Ok(exit_code) => exit_code,
    Err(message) => {
      compiler::error::print_message(&lazy, message);
      ExitCode::FAILURE
    },
  }
}

fn error_handler(
  lazy: &mut compiler::lazy::Lazy,
  verb: compiler::settings::Verb,
) -> Result<ExitCode, compiler::error::PrintableMessage> {
  match verb {
    compiler::settings::Verb::Check => {
      lazy.check()?;

      Ok(ExitCode::SUCCESS)
    },
    compiler::settings::Verb::Build => {
      lazy.build()?;

      Ok(ExitCode::SUCCESS)
    },
    compiler::settings::Verb::Run => {
      lazy.run()
    },
  }
}
