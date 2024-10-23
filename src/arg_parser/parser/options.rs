/// A command line flag; may be true or false
pub(super) enum Flag {
  Help,
}

/// A command line argument; holds a string value which is later validated.
/// Values are not checked until after each argument has been parsed.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy)]
pub(super) enum Argument {
  InputFile,
  OutputFile,
  CC,
  LLC,
}

/// Method for resolving an argument
pub(super) trait Process: Sized {
  /// Returns a variant of Self if argument represents Self
  fn from_argument(argument: &str) -> Option<Self>;
}

impl Process for Flag {
  fn from_argument(argument: &str) -> Option<Self> {
    match argument {
      "-h" | "--help" => Some(Self::Help),
      _ => None,
    }
  }
}

impl Process for Argument {
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

impl Argument {
  /// Used for printing errors: returns a string representation of this
  /// Argument
  pub(super) fn long_name(&self) -> &'static str {
    match self {
      Argument::InputFile => "--input",
      Argument::OutputFile => "--output",
      Argument::CC => "--cc",
      Argument::LLC => "--llc",
    }
  }
}
