mod format;
mod parsing;
mod info;

use std::path::PathBuf;
use std::process::ExitCode;

use crate::error::Level;

#[derive(Debug)]
pub struct Settings {
  pub executable: String,
  pub input_path: PathBuf,
  pub output_path: PathBuf,
  pub log_level: Level,
}

#[derive(Debug)]
pub enum Verb {
  Check,
  Build,
  Run,
}

#[derive(Debug)]
pub enum Error {
  Missing {
    what: &'static str,
    position: usize,
  },
  Invalid {
    what: &'static str,
    position: usize,
  },
  Verbless,
  Version,
  Help,
}

pub fn parse_and_display(mut argv: impl Iterator<Item = String>) -> Result<(Settings, Verb), ExitCode> {
  let Some(executable) = argv.next() else {
    eprintln!("\x1b[31merror\x1b[0m: argv is empty.  no process name was passed along.");
    info::help("{executable}");

    return Err(ExitCode::FAILURE);
  };

  let mut our_copy = vec![executable.clone()];

  let argv = argv.inspect(|argv| {
    our_copy.push(argv.to_owned());
  });

  match parsing::digest(&executable, argv) {
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
      format::show_error_position(our_copy, position);
      eprintln!();
      info::help(&executable);
      Err(ExitCode::FAILURE)
    },
    Err(Error::Missing { what, position }) => {
      eprint!("\x1b[31merror\x1b[0m: missing {what} at position #{position}:\n       ");
      format::show_error_position(our_copy, position);
      eprintln!();
      info::help(&executable);
      Err(ExitCode::FAILURE)
    },
  }
}
