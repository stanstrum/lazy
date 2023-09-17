/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::super::{
  SourceReader,
  AsterResult,
  ast::*,
  errors::*,
  consts,
  seek_read::seek
};

use super::try_make;

enum Base {
  Binary,
  Octal,
  Decimal,
  Hexadecimal,
}

enum StringType {
  C,
  Byte,
  Unicode
}

fn make_escape_sequence(reader: &mut SourceReader) -> AsterResult<char> {
  match reader.read_ch() {
    Ok('\'') => Ok('\''),
    Ok('"') => Ok('"'),
    Ok('\\') => Ok('\\'),
    Ok(consts::ascii::NL_ESCAPE) => Ok(consts::ascii::NL),
    Ok(consts::ascii::BL_ESCAPE) => Ok(consts::ascii::BL),
    Ok(consts::ascii::BS_ESCAPE) => Ok(consts::ascii::BS),
    Ok(consts::ascii::HT_ESCAPE) => Ok(consts::ascii::HT),
    Ok(consts::ascii::LF_ESCAPE) => Ok(consts::ascii::LF),
    Ok(consts::ascii::VT_ESCAPE) => Ok(consts::ascii::VT),
    Ok(consts::ascii::FF_ESCAPE) => Ok(consts::ascii::FF),
    Ok(consts::ascii::CR_ESCAPE) => Ok(consts::ascii::CR),
    Ok(consts::ascii::ES_ESCAPE) => Ok(consts::ascii::ES),
    Ok(consts::ascii::HEX_ESCAPE) => {
      todo!("hex escape")
    },
    Ok(consts::ascii::UNI_ESCAPE) => {
      todo!("unicode escape")
    },
    _ => {
      ExpectedSnafu {
        what: "Escape Sequence",
        offset: reader.offset()
      }.fail()
    }
  }
}

impl LiteralAST {
  pub fn make_numeric(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let base = {
      if seek::begins_with(reader, consts::punctuation::HEX_PFX) {
        Base::Hexadecimal
      } else if seek::begins_with(reader, consts::punctuation::BIN_PFX) {
        Base::Binary
      } else if seek::begins_with(reader, consts::punctuation::OCT_PFX) {
        Base::Octal
      } else {
        Base::Decimal
      }
    };

    let numeric_body_start = reader.offset();

    let mut decimal = false;
    #[allow(clippy::manual_is_ascii_check)]
    while let Some(ch) = reader.peek_ch() {
      match base {
        Base::Binary if matches!(ch, '0' | '1') => {},
        Base::Octal if matches!(ch, '0'..='7') => {},
        Base::Decimal if matches!(ch, '0'..='9') => {},
        Base::Decimal if !decimal && ch == '.' => {
          decimal = true;
        },
        Base::Hexadecimal if matches!(ch, '0'..='9' | 'A'..='F' | 'a'..='f') => {},
        _ if ch == '_' => {},
        _ => break
      };

      if reader.seek(1).is_err() {
        break;
      };
    };

    if reader.offset() == numeric_body_start {
      return ExpectedSnafu {
        what: "Numeric Literal",
        offset: reader.offset()
      }.fail();
    };

    let text = &reader.src()[start..reader.offset()];

    Ok(LiteralAST {
      span: reader.span_since(start),
      l: Literal::NumericLiteral(text.to_owned())
    })
  }

  pub fn make_string(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let ty = match reader.peek_ch() {
      Some('b') => {
        reader.seek(1).unwrap();

        StringType::Byte
      },
      Some('c') => {
        reader.seek(1).unwrap();

        StringType::C
      },
      Some('"') => StringType::Unicode,
      Some(_) | None => {
        return ExpectedSnafu {
          what: "String Literal",
          offset: reader.offset()
        }.fail()
      }
    };

    if !seek::begins_with(reader, consts::punctuation::QUOTE) {
      return ExpectedSnafu {
        what: "Quotation Mark",
        offset: reader.offset()
      }.fail();
    };

    let mut text = String::new();

    loop {
      let ch = match reader.read_ch() {
        Ok('"') => break,
        Ok('\\') => make_escape_sequence(reader)?,
        Ok(ch) => ch,
        Err(_) => {
          return ExpectedSnafu {
            what: "String Literal",
            offset: reader.offset()
          }.fail();
        },
      };

      text.push(ch);
    };

    let lit = match ty {
      StringType::Unicode => Literal::UnicodeString(text),
      StringType::Byte => Literal::ByteString(text),
      StringType::C => Literal::CString(text),
    };

    Ok(LiteralAST {
      span: reader.span_since(start),
      l: lit
    })
  }

  pub fn make_char(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let byte = match reader.peek_ch() {
      Some('b') => {
        reader.seek(1).unwrap();

        true
      },
      Some(_) => false,
      None => {
        return ExpectedSnafu {
          what: "Char Literal",
          offset: reader.offset()
        }.fail();
      }
    };

    if !seek::begins_with(reader, consts::punctuation::APOSTROPHE) {
      return ExpectedSnafu {
        what: "Single Quote",
        offset: reader.offset()
      }.fail();
    };

    let ch = match reader.read_ch() {
      Ok('\\') => make_escape_sequence(reader)?,
      Ok(ch) => ch,
      Err(_) => {
        return ExpectedSnafu {
          what: "Character",
          offset: reader.offset()
        }.fail();
      }
    };

    if !seek::begins_with(reader, consts::punctuation::APOSTROPHE) {
      return ExpectedSnafu {
        what: "Single Quote",
        offset: reader.offset()
      }.fail();
    };

    let lit = match byte {
      true => Literal::ByteChar(ch),
      false => Literal::Char(ch),
    };

    Ok(LiteralAST {
      span: reader.span_since(start),
      l: lit
    })
  }

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    if let Some(numeric) = try_make!(LiteralAST::make_numeric, reader) {
      Ok(numeric)
    } else if let Some(string) = try_make!(LiteralAST::make_string, reader) {
      Ok(string)
    } else if let Some(ch) = try_make!(LiteralAST::make_char, reader) {
      Ok(ch)
    } else {
      ExpectedSnafu {
        what: "Literal",
        offset: reader.offset()
      }.fail()
    }
  }
}
