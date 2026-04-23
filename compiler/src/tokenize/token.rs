use super::*;

use std::cmp::Ordering;

use crate::lang::reference::ModuleReference;
use crate::lang::ty::Intrinsic;

pub use ::token::span::Position;
pub type Span = ::token::span::ModuleSpan<ModuleReference>;
pub use ::token::special::*;
pub use ::token::{CharKind, StringKind, Token};

pub type TokenSpan = (Token, Span);

#[derive(Debug)]
pub struct StringState {
  pub content: String,
  pub kind: StringKind,
  pub start: Position,
}

#[derive(Debug, Clone, Copy)]
pub struct CharState {
  pub ch: Option<char>,
  pub kind: CharKind,
  pub start: Position,
}

#[derive(Debug)]
pub enum EscapeReturn {
  String(StringState),
  Char(CharState),
}

#[derive(Debug)]
pub enum EscapeValue {
  Char(char),
  ReadHex,
  ReadOctal,
  Unicode,
}

impl From<StringKind> for Intrinsic {
  fn from(value: StringKind) -> Self {
    match value {
      StringKind::Wide => Self::U32,
      StringKind::Byte | StringKind::C => Self::U8,
    }
  }
}

impl From<EscapeReturn> for State {
  fn from(value: EscapeReturn) -> Self {
    match value {
      EscapeReturn::String(string_state) => Self::String(string_state),
      EscapeReturn::Char(char_state) => Self::Char(char_state),
    }
  }
}

impl EscapeReturn {
  pub fn append_ch(&mut self, ch: char) {
    match self {
      EscapeReturn::String(StringState { content, .. }) => {
        content.push(ch);
      },
      EscapeReturn::Char(CharState { ch: option, .. }) => {
        assert!(option.is_none());
        *option = Some(ch);
      },
    }
  }
}

pub fn parse_escape(value: &str) -> Result<EscapeValue, Error> {
  match value {
    "0" => return Ok(EscapeValue::Char('\0')),
    "a" => return Ok(EscapeValue::Char('\x07')),
    "b" => return Ok(EscapeValue::Char('\x08')),
    "t" => return Ok(EscapeValue::Char('\t')),
    "n" => return Ok(EscapeValue::Char('\n')),
    "v" => return Ok(EscapeValue::Char('\x0b')),
    "f" => return Ok(EscapeValue::Char('\x0c')),
    "r" => return Ok(EscapeValue::Char('\r')),
    "e" => return Ok(EscapeValue::Char('\x1b')),
    _ => {},
  };

  if let Some(value) = value.strip_prefix("x") {
    if !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
      panic!("invalid hex");
    };

    match value.len().cmp(&2) {
      Ordering::Greater => panic!("too many bytes"),
      Ordering::Equal => {
        let Ok(byte) = u8::from_str_radix(value, 16) else {
          panic!("invalid hex (should never panic)");
        };

        return Ok(EscapeValue::Char(byte as char));
      },
      Ordering::Less => {
        return Ok(EscapeValue::ReadHex);
      },
    };
  };

  if let Some(value) = value.strip_prefix("o") {
    if !value.chars().all(|ch| matches!(ch, '0'..='8')) {
      panic!("invalid octal");
    };

    match value.len().cmp(&2) {
      Ordering::Greater => panic!("too many bytes"),
      Ordering::Equal => {
        let Ok(byte) = u8::from_str_radix(value, 8) else {
          panic!("invalid octal (should never panic)");
        };

        return Ok(EscapeValue::Char(byte as char));
      },
      Ordering::Less => {
        return Ok(EscapeValue::ReadOctal);
      },
    };
  };

  if let Some(_value) = value.strip_prefix("u") {
    todo!("unicode");
  };

  todo!();
}
