mod seek;
mod print;

use std::io::{BufRead, BufReader, Write};
use std::fs::File;

use crate::lang::module::ModuleId;
use crate::{lang::Lazy, tokenize::token::Span};

pub use print::print_message;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Level {
  Debug,
  Info,
  Warn,
  Error,
}

#[derive(Debug)]
pub struct PrintableMesage {
  pub level: Level,
  pub force: bool,
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
    write!(f, "{self:?}")
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
        force: true,
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
        force: true,
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
        force: true,
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
