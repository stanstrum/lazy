pub(super) enum Flag {
  Help,
}

#[derive(Clone, Copy)]
pub(super) enum Argument {
  InputFile,
  OutputFile,
  CC,
  LLC,
}

pub(super) trait Process: Sized {
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
  pub(super) fn long_name(&self) -> &'static str {
    match self {
      Argument::InputFile => "--input",
      Argument::OutputFile => "--output",
      Argument::CC => "--cc",
      Argument::LLC => "--llc",
    }
  }
}
