mod options;

use crate::{Result, ok};
use super::error::*;
use options::*;

enum State {
  First,
  ArgumentFlag(Argument),
}

pub(super) struct CompilerParser {
  state: State,
  pub(super) help: bool,
  pub(super) input_file: Option<String>,
  pub(super) output_file: Option<String>,
  pub(super) llc: Option<String>,
  pub(super) cc: Option<String>,
}

impl CompilerParser {
  pub(super) fn new() -> Self {
    Self {
      state: State::First,
      help: false,
      input_file: None,
      output_file: None,
      llc: None,
      cc: None,
    }
  }

  fn string_pointer(&mut self, kind: Argument) -> &mut Option<String> {
    match kind {
      Argument::InputFile => &mut self.input_file,
      Argument::OutputFile => &mut self.output_file,
      Argument::CC => &mut self.cc,
      Argument::LLC => &mut self.llc,
    }
  }

  fn set_option_string_value(&mut self, kind: Argument, argument: String) -> Result {
    let option = self.string_pointer(kind);

    if let Some(original) = option {
      let long_name = kind.long_name();

      warn!("\
        duplicate argument values:\n  \
          {long_name}={original:?}\n  \
          {long_name}={argument:?}\
      ");

      return DuplicateSnafu { long_name }.fail()?;
    };

    *option = Some(argument);

    ok
  }

  fn first(&mut self, argument: String) -> Result {
    if let Some(flag) = Flag::from_argument(&argument) {
      match flag {
        Flag::Help => self.help = true,
      };

      return ok;
    };

    let (key, value) = match argument.split_once("=") {
      Some((key, value)) => (key, Some(value)),
      None => (argument.as_str(), None),
    };

    if let Some(kind) = Argument::from_argument(key) {
      return if let Some(value) = value {
        self.set_option_string_value(kind, value.into())
      } else {
        self.state = State::ArgumentFlag(kind);

        ok
      };
    };

    // input file is the implicit first argument
    if self.input_file.is_none() {
      self.input_file = Some(argument);

      return ok;
    };

    UnknownFlagSnafu { flag: argument }.fail()?
  }

  fn argument_flag(&mut self, kind: Argument, argument: String) -> Result {
    self.set_option_string_value(kind, argument)?;
    self.state = State::First;

    ok
  }

  pub(super) fn parse_argument(&mut self, argument: String) -> Result {
    match self.state {
      State::First => self.first(argument),
      State::ArgumentFlag(kind) => self.argument_flag(kind, argument),
    }
  }
}
