mod info;

use std::process::ExitCode;
use std::path::PathBuf;

use compiler::error::Level;
use compiler::settings::Settings;

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

pub(super) fn parse_and_display(mut argv: impl Iterator<Item = String>) -> Result<(Settings, Verb), ExitCode> {
  let Some(executable) = argv.next() else {
    eprintln!("\x1b[31merror\x1b[0m: argv is empty.  no process name was passed along.");
    info::help("{executable}");

    return Err(ExitCode::FAILURE);
  };

  let mut our_copy = vec![executable.clone()];

  let argv = argv.inspect(|argv| {
    our_copy.push(argv.to_owned());
  });

  match digest(&executable, argv) {
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

fn parse_log_level(level: String) -> Option<Level> {
  match level.to_lowercase().as_str() {
    "debug" | "dbg" | "d" => Some(Level::Debug),
    "error" | "err" | "e" => Some(Level::Error),
    "warn"  | "wrn" | "w" => Some(Level::Warn),
    "info"  | "inf" | "i" => Some(Level::Info),
    _ => None,
  }
}

fn digest(executable: &str, argv: impl Iterator<Item = String>) -> Result<(Settings, Verb), Error> {
  let mut input_path = None;
  let mut log_level = None;
  let mut output_path = None;

  let mut position = 0;
  let mut result = vec![executable.to_owned()];
  let mut argv = argv.inspect(|arg| {
    result.push(arg.to_owned());
    position += 1;
  });

  let executable = executable.to_owned();

  let mut verb = None;

  loop {
    let Some(option) = argv.next() else {
      break;
    };

    match option.as_str() {
      "check" | "ck" | "c" if verb.is_none() => verb = Some(Verb::Check),
      "run"   | "r"        if verb.is_none() => verb = Some(Verb::Run),
      "build" | "b"        if verb.is_none() => verb = Some(Verb::Build),
      "--version"     | "-v" => return Err(Error::Version),
      "--help"        | "-h" => return Err(Error::Help),
      "--log-level"   | "--log"    | "-l" => {
        let what: &'static str = "parameter for log level";

        let Some(level) = argv.next() else {
          return Err(Error::Missing { what, position });
        };

        let Some(level) = parse_log_level(level) else {
          return Err(Error::Invalid { what, position });
        };

        log_level = Some(level);
      },
      _ if (
        option.starts_with("--log-level=") ||
        option.starts_with("--log=") ||
        option.starts_with("-l=")
      ) => {
        let what: &'static str = "parameter for log level";

        let (_, level) = option.split_once('=')
          .expect("option to have equals sign");

        let Some(level) = parse_log_level(level.to_owned()) else {
          return Err(Error::Invalid { what, position });
        };

        log_level = Some(level);
      },
      "--output-file" | "--output" | "-o" => {
        let Some(path) = argv.next() else {
          return Err(Error::Missing {
            what: "parameter for output path",
            position,
          });
        };

        output_path = Some(PathBuf::from(path));
      },
      _ if (
        option.starts_with("--output-file=") ||
        option.starts_with("--output=") ||
        option.starts_with("-o")
       ) => {
        let (_, path) = option.split_once('=')
          .expect("option to have equals sign");

        output_path = Some(PathBuf::from(path));
      },
      _ if input_path.is_none() && verb.is_some() => input_path = Some(PathBuf::from(option)),
      _ => return Err(Error::Invalid {
        what: "argument",
        position,
      }),
    };
  };

  let verb = verb.ok_or(Error::Verbless)?;

  let input_path = input_path.ok_or(Error::Missing {
    what: "input path",
    position,
  })?;

  let output_path = output_path.unwrap_or_else(|| PathBuf::from("./a.out"));
  let log_level = log_level.unwrap_or(Level::Info);

  let settings = Settings {
    executable,
    input_path,
    output_path,
    log_level,
    argv: result,
  };

  Ok((settings, verb))
}
