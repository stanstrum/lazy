mod seek;
mod print;

use std::io::{BufRead, BufReader, Write};
use std::fs::File;

use crate::colorize;
use crate::lang::module::ModuleId;
use crate::{lang::Lazy, tokenize::token::Span};

pub use print::print_message;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
#[allow(unused)]
pub enum Level {
  Debug,
  Info,
  Warn,
  Error,
}

#[derive(Debug)]
pub struct PrintableMesage {
  pub level: Level,
  pub _force: bool,
  pub description: String,
  pub contents: MessageContents,
}

#[derive(Debug)]
pub enum MessageContents {
  WithinSource {
    range: Span,
    sections: Vec<MessageSection>,
  },
  File(ModuleId),
}

#[derive(Debug)]
pub struct MessageSection {
  pub text: String,
  pub span: Span,
}

impl std::fmt::Display for Level {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      Level::Debug => concat!(colorize!(7), colorize!(3), "debug", colorize!(0)),
      Level::Info => concat!(colorize!(7), colorize!(92), "info", colorize!(0)),
      Level::Warn => concat!(colorize!(7), colorize!(93), "warn", colorize!(0)),
      Level::Error => concat!(colorize!(7), colorize!(91), "error", colorize!(0)),
    })
  }
}

impl From<crate::tokenize::Error> for PrintableMesage {
  fn from(value: crate::tokenize::Error) -> Self {
    match value {
      crate::tokenize::Error::IO => todo!(),
      crate::tokenize::Error::InvalidNumeric => todo!(),
    }
  }
}

impl From<crate::aster::Error> for PrintableMesage {
  fn from(value: crate::aster::Error) -> Self {
    match value {
      crate::aster::Error::Token(error) => error.into(),
      crate::aster::Error::Expected { what, at } => Self {
        level: Level::Error,
        _force: true,
        description: format!("expected {what}"),
        contents: MessageContents::WithinSource {
          range: at,
          sections: vec![MessageSection {
            text: "here".into(),
            span: at,
          }],
        },
      },
      crate::aster::Error::Invalid { what, at } => Self {
        level: Level::Error,
        _force: true,
        description: format!("invalid {what}"),
        contents: MessageContents::WithinSource {
          range: at,
          sections: vec![MessageSection {
            text: "here".into(),
            span: at,
          }],
        },
      },
    }
  }
}

impl From<crate::resolve::verify::Error> for PrintableMesage {
  fn from(value: crate::resolve::verify::Error) -> Self {
    match value {
      crate::resolve::verify::Error::Unresolved { what, at } => Self {
        level: Level::Error,
        _force: true,
        description: format!("verify: unresolved {what}"),
        contents: MessageContents::WithinSource {
          range: at,
          sections: vec![MessageSection {
            text: "here".into(),
            span: at,
          }],
        },
      },
    }
  }
}

impl From<crate::resolve::Error> for PrintableMesage {
  fn from(value: crate::resolve::Error) -> Self {
    match value {
      crate::resolve::Error::Verify(error) => error.into(),
    }
  }
}
