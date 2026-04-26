mod parse;
mod info;

use std::process::ExitCode;

use gluezy::Settings;
use crate::parse::{Verb, Error};

/// The entry point for the command-line interface to the LaZY compiler.
fn main() -> ExitCode {
  // Get argv
  let mut argv = std::env::args();

  // Get the name of the executable used to invoke the compiler
  let Some(executable) = argv.next() else {
    eprintln!("\x1b[31merror\x1b[0m: argv is empty.  no process name was passed along.");
    info::help("{executable}");

    return ExitCode::FAILURE;
  };

  // Keep a copy of argv if there's an error and we need to print it.  I have
  // the feeling that there was a reason I wanted to do it this way.  Wasn't a
  // good reason, though
  let mut our_copy_of_argv = vec![executable.clone()];

  let argv = argv.inspect(|argv| {
    our_copy_of_argv.push(argv.to_owned());
  });

  // Parse and handle CLI arguments: either return some settings for the compiler
  // or, for example, print help message
  let (settings, verb) = match parse::digest(&executable, argv) {
    Ok(settings) => settings,
    Err(Error::Version) => return info::version(),
    Err(Error::Help | Error::Verbless) => return info::help(&executable),
    Err(Error::Invalid { what, position }) => {
      eprint!("\x1b[31merror\x1b[0m: invalid {what} at position #{position}:\n       ");

      gluezy::format::show_error_position(our_copy_of_argv, position);
      eprintln!();

      return info::help(&executable);
    },
    Err(Error::Missing { what, position }) => {
      eprint!("\x1b[31merror\x1b[0m: missing {what} at position #{position}:\n       ");

      gluezy::format::show_error_position(our_copy_of_argv, position);
      eprintln!();

      return info::help(&executable);
    },
  };

  // Call the compiler with the parsed arguments
  lazy(settings, verb)
}

/// This function takes in [`Settings`] and a [`Verb`] and sets up an empty
/// context and runs the compiler with the specified instructions and options.
/// This is the function called by the CLI after parsing arguments from argv.
pub fn lazy(settings: Settings, verb: Verb) -> ExitCode {
  let pool = compiler::StringPool::new();
  let lazy = &mut gluezy::Lazy::new(&pool, settings);

  let result = match verb {
    parse::Verb::Check => compiler::check(lazy).and(Ok(ExitCode::SUCCESS)),
    parse::Verb::Build => compiler::build(lazy).and(Ok(ExitCode::SUCCESS)),
    parse::Verb::Run => compiler::run(lazy),
  };

  match result {
    Ok(exit_code) => exit_code,
    Err(message) => {
      log::print_message(lazy, message);
      ExitCode::FAILURE
    },
  }
}
