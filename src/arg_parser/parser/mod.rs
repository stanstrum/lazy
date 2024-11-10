mod options;

use crate::{Result, ok};
use super::error::*;
use options::*;

enum State {
  /// Beginning of a flag or an implicit entry point
  First,
  /// Second half of an argument delimited by a space
  ArgumentFlag(Argument),
}

/// Parses command-line arguments for the compiler
pub(super) struct CompilerParser {
  state: State,
  /// Show help text
  pub(super) help: bool,
  /// Entry point for compilation, required
  pub(super) input_file: Option<String>,
  /// Output path for executable, optional
  pub(super) output_file: Option<String>,
  /// Path to LLC executable
  pub(super) llc: Option<String>,
  /// Path to CC executable
  pub(super) cc: Option<String>,
  /// Print LLVM code during generation
  pub(super) print_llvm: bool,
}

impl CompilerParser {
  /// Makes a CompilerParser
  pub(super) fn new() -> Self {
    Self {
      state: State::First,
      help: false,
      input_file: None,
      output_file: None,
      llc: None,
      cc: None,
      print_llvm: false,
    }
  }

  /// Returns a pointer to the option in Self that stores the value of the
  /// corresponding Argument
  fn string_pointer(&mut self, kind: Argument) -> &mut Option<String> {
    match kind {
      Argument::InputFile => &mut self.input_file,
      Argument::OutputFile => &mut self.output_file,
      Argument::CC => &mut self.cc,
      Argument::LLC => &mut self.llc,
    }
  }

  /// Saves the value of the provided Argument in Self
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

  /// Process the former half of an argument
  fn first(&mut self, argument: String) -> Result {
    // Check if this argument is a valid flag
    if let Some(flag) = Flag::from_argument(&argument) {
      match flag {
        Flag::Help => self.help = true,
        Flag::PrintLLVM => self.print_llvm = true,
      };

      return ok;
    };

    // Split argument on first equals sign
    let (key, value) = match argument.split_once("=") {
      Some((key, value)) => (key, Some(value)),
      None => (argument.as_str(), None),
    };

    // Check if this argument is a valid argument
    if let Some(kind) = Argument::from_argument(key) {
      return if let Some(value) = value {
        // If we had a value already in this argument (with an equals sign),
        // then set the value accordingly and continue onto parsing the next
        // argument immediately
        self.set_option_string_value(kind, value.into())
      } else {
        // Otherwise, go on to parse the value for this argument in the next
        // argument
        self.state = State::ArgumentFlag(kind);

        ok
      };
    };

    // Otherwise, input file is the implicit argument
    if self.input_file.is_none() {
      self.input_file = Some(argument);

      return ok;
    };

    // Fallthrough: unrecognized flag or argument
    UnknownFlagSnafu { flag: argument }.fail()?
  }

  /// Parses one argument as provided and updates internal state accordingly
  pub(super) fn parse_argument(&mut self, argument: String) -> Result {
    match self.state {
      State::First => self.first(argument),
      State::ArgumentFlag(kind) => {
        self.set_option_string_value(kind, argument)?;
        self.state = State::First;

        ok
      }
    }
  }
}
