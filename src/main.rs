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
  let mut lazy = compiler::Lazy::new(&pool, settings);

  match error_handler(&mut lazy, verb) {
    Ok(exit_code) => exit_code,
    Err(message) => {
      compiler::error::print_message(&lazy, message);
      ExitCode::FAILURE
    },
  }
}

fn error_handler(
  lazy: &mut compiler::Lazy,
  verb: parse::Verb,
) -> Result<ExitCode, compiler::error::PrintableMessage> {
  match verb {
    parse::Verb::Check => {
      lazy.check()?;

      Ok(ExitCode::SUCCESS)
    },
    parse::Verb::Build => {
      lazy.build()?;

      Ok(ExitCode::SUCCESS)
    },
    parse::Verb::Run => {
      lazy.run()
    },
  }
}
