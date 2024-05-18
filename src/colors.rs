#[allow(unused)]
#[derive(Debug)]
pub(crate) enum Color {
  Black,
  Red,
  Green,
  Yellow,
  Blue,
  Magenta,
  Cyan,
  LightGrey,
  DarkGrey,
  LightRed,
  LightGreen,
  LightYellow,
  LightBlue,
  LightMagenta,
  LightCyan,
  White,

  Creme,
  Teal,
  Mint,

  Underline,
  Bold,
  Clear,
}

impl std::string::ToString for Color {
  fn to_string(&self) -> String {
    match self {
      Self::Black => "\x1b[30m",
      Self::Red => "\x1b[31m",
      Self::Green => "\x1b[32m",
      Self::Yellow => "\x1b[33m",
      Self::Blue => "\x1b[34m",
      Self::Magenta => "\x1b[35m",
      Self::Cyan => "\x1b[36m",
      Self::LightGrey => "\x1b[37m",
      Self::DarkGrey => "\x1b[90m",
      Self::LightRed => "\x1b[91m",
      Self::LightGreen => "\x1b[92m",
      Self::LightYellow => "\x1b[38;5;215m",
      Self::LightBlue => "\x1b[38;5;159m",
      Self::LightMagenta => "\x1b[95m",
      Self::LightCyan => "\x1b[96m",
      Self::White => "\x1b[97m",
      Self::Creme => "\x1b[38;5;230m",
      Self::Teal => "\x1b[38;5;159m",
      Self::Mint => "\x1b[38;5;158m",
      Self::Underline => "\x1b[4m",
      Self::Bold => "\x1b[1m",
      Self::Clear => "\x1b[0m",
    }.to_string()
  }
}
