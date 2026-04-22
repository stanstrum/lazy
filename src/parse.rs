use std::process::ExitCode;

use compiler::settings::{Error, Settings, Verb};

pub(super) fn parse_and_display(mut argv: impl Iterator<Item = String>) -> Result<(Settings, Verb), ExitCode> {
  let Some(executable) = argv.next() else {
    eprintln!("\x1b[31merror\x1b[0m: argv is empty.  no process name was passed along.");
    crate::info::help("{executable}");

    return Err(ExitCode::FAILURE);
  };

  let mut our_copy = vec![executable.clone()];

  let argv = argv.inspect(|argv| {
    our_copy.push(argv.to_owned());
  });

  match compiler::settings::parsing::digest(&executable, argv) {
    Ok(settings) => Ok(settings),
    Err(Error::Version) => {
      crate::info::version();
      Err(ExitCode::FAILURE)
    },
    Err(Error::Help | Error::Verbless) => {
      crate::info::help(&executable);
      Err(ExitCode::FAILURE)
    },
    Err(Error::Invalid { what, position }) => {
      eprint!("\x1b[31merror\x1b[0m: invalid {what} at position #{position}:\n       ");
      compiler::settings::format::show_error_position(our_copy, position);
      eprintln!();
      crate::info::help(&executable);
      Err(ExitCode::FAILURE)
    },
    Err(Error::Missing { what, position }) => {
      eprint!("\x1b[31merror\x1b[0m: missing {what} at position #{position}:\n       ");
      compiler::settings::format::show_error_position(our_copy, position);
      eprintln!();
      crate::info::help(&executable);
      Err(ExitCode::FAILURE)
    },
  }
}
