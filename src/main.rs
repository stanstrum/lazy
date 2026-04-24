mod parse;
mod info;

use std::process::ExitCode;

use parse::Error;

fn main() -> ExitCode {
  let args = std::env::args();
  run_with(args.into_iter())
}

fn run_with(args: impl Iterator<Item = String>) -> ExitCode {
  let (settings, verb) = match parse_and_display(args) {
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

fn parse_and_display(mut argv: impl Iterator<Item = String>) -> Result<
  (compiler::settings::Settings, parse::Verb), ExitCode
> {
  let Some(executable) = argv.next() else {
    eprintln!("\x1b[31merror\x1b[0m: argv is empty.  no process name was passed along.");
    info::help("{executable}");

    return Err(ExitCode::FAILURE);
  };

  let mut our_copy = vec![executable.clone()];

  let argv = argv.inspect(|argv| {
    our_copy.push(argv.to_owned());
  });

  match parse::digest(&executable, argv) {
    Ok(settings) => Ok(settings),
    Err(Error::Version) => {
      info::version();
      Err(ExitCode::FAILURE)
    },
    Err(Error::Help | Error::Verbless) => {
      info::help(&executable);
      Err(ExitCode::FAILURE)
    },
    Err(Error::Invalid { what, position }) => {
      eprint!("\x1b[31merror\x1b[0m: invalid {what} at position #{position}:\n       ");
      compiler::format::show_error_position(our_copy, position);
      eprintln!();
      info::help(&executable);
      Err(ExitCode::FAILURE)
    },
    Err(Error::Missing { what, position }) => {
      eprint!("\x1b[31merror\x1b[0m: missing {what} at position #{position}:\n       ");
      compiler::format::show_error_position(our_copy, position);
      eprintln!();
      info::help(&executable);
      Err(ExitCode::FAILURE)
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
