use std::char;

use crate::tokenizer::{
  Tokenizer,
  PeekReader,
  Result,
  error::*,
};

impl Tokenizer {
  pub(super) fn hexadecimal_escape(&mut self, reader: &mut PeekReader) -> Result<char> {
    let text = reader
      .take(2)
      .map(|item| Ok(item?.ch))
      .collect::<Result<String>>()?;

    let Ok(value) = u8::from_str_radix(&text, 16) else {
      return InvalidSnafu {
        what: What::StringEscapeSequence,
        content: &text,
      }.fail()?;
    };

    Ok(value as char)
  }

  pub(super) fn unicode_escape(&mut self, reader: &mut PeekReader) -> Result<char> {
    reader.starts_with_seek("{")?;

    let mut text = String::new();

    for _ in 0..=4 {
      let Some(item) = reader.next() else {
        return ExpectedSnafu {
          what: What::StringEscapeSequence,
        }.fail()?;
      };

      let item = item?;

      if item.ch == '}' {
        break;
      };

      text.push(item.ch);
    };

    let Ok(value) = u32::from_str_radix(&text, 16) else {
      return InvalidSnafu {
        what: What::StringEscapeSequence,
        content: &text,
      }.fail()?;
    };

    let Some(character) = char::from_u32(value) else {
      return InvalidSnafu {
        what: What::StringEscapeSequence,
        content: &text,
      }.fail()?;
    };

    Ok(character)
  }
}
