mod escapes;

use crate::{Result, ok};
use crate::tokenizer::{
  PeekReader,
  TokenKind,
  Tokenizer,
  error::*,
};

#[derive(Debug)]
enum NumericEscape {
  HexadecimalEscape,
  UnicodeEscape,
}

enum EscapedCharacter {
  Null,
  Bell,
  Backspace,
  Tab,
  Newline,
  VerticalTab,
  FormFeed,
  CarriageReturn,
  Escape,
  Backslash,
  DoubleQuote,
  SingleQuote,
  SkippedMultiline,
  NumericEscape(NumericEscape),
}

impl EscapedCharacter {
  /// Returns a variant of Self if ch is recognized as an escape code
  fn from_char(ch: char) -> Option<Self> {
    match ch {
      '0' => Some(Self::Null),
      'a' => Some(Self::Bell),
      'b' => Some(Self::Backspace),
      't' => Some(Self::Tab),
      'n' => Some(Self::Newline),
      'v' => Some(Self::VerticalTab),
      'f' => Some(Self::FormFeed),
      'r' => Some(Self::CarriageReturn),
      'e' => Some(Self::Escape),
      '\\' => Some(Self::Backslash),
      '"' => Some(Self::DoubleQuote),
      '\'' => Some(Self::SingleQuote),
      '\n' => Some(Self::SkippedMultiline),
      'x' => Some(Self::NumericEscape(NumericEscape::HexadecimalEscape)),
      'u' => Some(Self::NumericEscape(NumericEscape::UnicodeEscape)),
      _ => None,
    }
  }

  /// Returns a string representation of this escaped value if possible
  fn as_escaped_value(&self) -> Option<char> {
    match self {
      Self::Null => Some('\0'),
      Self::Bell => Some('\x07'),
      Self::Backspace => Some('\x08'),
      Self::Tab => Some('\t'),
      Self::Newline => Some('\n'),
      Self::VerticalTab => Some('\x0b'),
      Self::FormFeed => Some('\x0c'),
      Self::CarriageReturn => Some('\r'),
      Self::Escape => Some('\x1b'),
      Self::Backslash => Some('\\'),
      Self::DoubleQuote => Some('"'),
      Self::SingleQuote => Some('\''),
      | Self::SkippedMultiline
      | Self::NumericEscape(_) => None,
    }
  }
}

impl Tokenizer {
  fn numeric_escape(&mut self, reader: &mut PeekReader, escape: NumericEscape) -> Result<char> {
    match escape {
      NumericEscape::HexadecimalEscape => self.hexadecimal_escape(reader),
      NumericEscape::UnicodeEscape => self.unicode_escape(reader),
    }
  }

  fn text_escape(&mut self, reader: &mut PeekReader) -> Result<Option<char>> {
    let Some(item) = reader.next() else {
      return ExpectedSnafu { what: What::String }.fail()?;
    };

    let ch = item?.ch;

    let Some(escaped) = EscapedCharacter::from_char(ch) else {
      return InvalidSnafu {
        what: What::String,
        content: format!("\\{ch}"),
      }.fail()?;
    };

    match escaped {
      EscapedCharacter::NumericEscape(escape) => {
        self.numeric_escape(reader, escape)
          .map(Some)
      },
      escaped => Ok(
        escaped.as_escaped_value()
      ),
    }
  }

  fn text_character(&mut self, reader: &mut PeekReader, end: char) -> Result<Option<char>> {
    let Some(item) = reader.next() else {
      return ExpectedSnafu { what: What::String }.fail()?;
    };

    Ok({
      match item?.ch {
        '\\' => self.text_escape(reader)?,
        ch => (ch != end).then_some(ch),
      }
    })
  }

  fn string_content(&mut self, reader: &mut PeekReader, quote: char) -> Result {
    let start = reader.span_start();
    let mut content = String::new();

    if !reader.starts_with_seek(String::from(quote).as_str())? {
      return ExpectedSnafu { what: What::String }.fail()?;
    };

    while let Some(ch) = self.text_character(reader, quote)? {
      content.push(ch);
    };

    self.push_tok(TokenKind::String(content), start, reader.position);

    ok
  }

  pub(in crate::tokenizer) fn string(&mut self, reader: &mut PeekReader) -> Result {
    self.string_content(reader, '"')
  }

  pub(in crate::tokenizer) fn char(&mut self, reader: &mut PeekReader) -> Result {
    self.string_content(reader, '\'')
  }
}
