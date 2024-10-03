use std::path::PathBuf;
use std::str::FromStr;

use which::which;

enum CompilerParserFlag {
  Help,
}

#[derive(Clone, Copy)]
enum CompilerParserArgument {
  InputFile,
  OutputFile,
  CC,
  LLC,
}

trait CompilerParserProcess: Sized {
  fn from_argument(argument: &str) -> Option<Self>;
}

enum CompilerParserState {
  First,
  ArgumentFlag(CompilerParserArgument),
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct CompilerOptions {
  pub(crate) help: bool,
  pub(crate) input_file: Option<PathBuf>,
  pub(crate) output_file: PathBuf,
  pub(crate) llc: PathBuf,
  pub(crate) cc: PathBuf,
}

pub(crate) struct CompilerParser {
  state: CompilerParserState,
  help: bool,
  input_file: Option<String>,
  output_file: Option<String>,
  llc: Option<String>,
  cc: Option<String>,
}

impl CompilerParserProcess for CompilerParserFlag {
  fn from_argument(argument: &str) -> Option<Self> {
    match argument {
      "-h" | "--help" => Some(Self::Help),
      _ => None,
    }
  }
}

impl CompilerParserProcess for CompilerParserArgument {
  fn from_argument(argument: &str) -> Option<Self> {
    match argument {
      "-i" | "--input" => Some(Self::InputFile),
      "-o" | "--output" => Some(Self::OutputFile),
      "--llc" => Some(Self::LLC),
      "--cc" => Some(Self::CC),
      _ =>  None,
    }
  }
}

impl CompilerParserArgument {
  fn long_name(&self) -> &'static str {
    match self {
      CompilerParserArgument::InputFile => "--input",
      CompilerParserArgument::OutputFile => "--output",
      CompilerParserArgument::CC => "--cc",
      CompilerParserArgument::LLC => "--llc",
    }
  }
}

fn default_option_resolve_path_strerror(path: Option<String>, default: &'static str) -> Result<PathBuf, String>  {
  let path = path.as_ref()
    .map(|path| path.as_str())
    .unwrap_or(default);

  which(path)
    .map_err(|err| format!(
      "{path}: failed to locate ({err})"
    ))
}

impl TryFrom<CompilerParser> for CompilerOptions {
  type Error = String;

  fn try_from(parser: CompilerParser) -> Result<Self, Self::Error> {
    let CompilerParser {
      help,
      input_file,
      output_file,
      llc,
      cc,
      ..
    } = parser;

    let input_file = if let Some(input_file) = input_file {
      let input_file = PathBuf::from_str(&input_file).unwrap();

      match std::fs::canonicalize(&input_file) {
        Ok(x) => Some(x),
        Err(err) => return Err(
          format!(
            "{}: could not locate: {err}",
            input_file.to_string_lossy()
          )
        ),
      }
    } else {
      None
    };

    let output_file = output_file.unwrap_or("a.out".into());
    let output_file = PathBuf::from_str(&output_file).unwrap();

    let llc = default_option_resolve_path_strerror(llc, "llc")?;
    let cc = default_option_resolve_path_strerror(cc, "cc")?;

    Ok(Self {
      help,
      input_file,
      output_file,
      llc,
      cc,
    })
  }
}

#[allow(non_upper_case_globals)]
const ok: Result<(), String> = Ok(());

impl CompilerParser {
  fn new() -> Self {
    Self {
      state: CompilerParserState::First,
      help: false,
      input_file: None,
      output_file: None,
      llc: None,
      cc: None,
    }
  }

  fn string_pointer(&mut self, kind: CompilerParserArgument) -> &mut Option<String> {
    match kind {
      CompilerParserArgument::InputFile => &mut self.input_file,
      CompilerParserArgument::OutputFile => &mut self.output_file,
      CompilerParserArgument::CC => &mut self.cc,
      CompilerParserArgument::LLC => &mut self.llc,
    }
  }

  fn set_option_string_value_or_strerror(option: &mut Option<String>, kind: CompilerParserArgument, argument: String) -> Result<(), String> {
    if let Some(original) = option {
      let long_name = kind.long_name();

      warn!("\
        duplicate argument values:\n  \
          {long_name}={original:?}\n  \
          {long_name}={argument:?}\
      ");

      return Err(format!("already received argument: {long_name}"));
    };

    *option = Some(argument);

    ok
  }

  fn first(&mut self, argument: String) -> Result<(), String> {
    if let Some(flag) = CompilerParserFlag::from_argument(&argument) {
      match flag {
        CompilerParserFlag::Help => self.help = true,
      };
    } else {
      let (key, value) = match argument.split_once("=") {
        Some((key, value)) => (key, Some(value)),
        None => (argument.as_str(), None),
      };

      if let Some(kind) = CompilerParserArgument::from_argument(key) {
        match value {
          Some(value) => {
            Self::set_option_string_value_or_strerror(
              self.string_pointer(kind),
              kind,
              value.into()
            )?;
          },
          None => {
            self.state = CompilerParserState::ArgumentFlag(kind);
          },
        };
      } else if self.input_file.is_none() {
        self.input_file = Some(argument);
      } else  {
        return Err(format!("unrecognized flag: {argument:?}"));
      }
    };

    ok
  }

  fn argument_flag(&mut self, kind: CompilerParserArgument, argument: String) -> Result<(), String> {
    Self::set_option_string_value_or_strerror(
      self.string_pointer(kind),
      kind,
      argument,
    )?;

    self.state = CompilerParserState::First;

    ok
  }

  fn parse_argument(&mut self, argument: String) -> Result<(), String> {
    match self.state {
      CompilerParserState::First => self.first(argument),
      CompilerParserState::ArgumentFlag(kind) => self.argument_flag(kind, argument),
    }
  }
}

pub(crate) fn parse() -> Result<CompilerOptions, String> {
  let mut parser = CompilerParser::new();

  for argument in std::env::args().skip(1) {
    parser.parse_argument(argument)?;
  };

  CompilerOptions::try_from(parser)
}
