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

impl Literal {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let first = match reader.peek_ch() {
      Some('b') => {
        reader.seek(1).unwrap();

        Some('b')
      },
      Some('"') => None,
      Some(_) | None => {
        return ExpectedSnafu {
          what: "Byte String",
          offset: reader.offset()
        }.fail();
      }
    };

    let l = Literal::make_string_body(reader)?;

    match first {
      Some('b') => Ok(Literal::ByteString(l)),
      None => Ok(Literal::String(l)),
      _ => panic!("invalid string prefix retained")
    }
  }

  pub fn make_string_body(reader: &mut SourceReader) -> AsterResult<String> {
    if !seek::begins_with(reader, consts::punctuation::QUOTE) {
      return ExpectedSnafu {
        what: "Quote",
        offset: reader.offset()
      }.fail();
    };

    let mut text = String::new();

    loop {
      let Ok(ch) = reader.read(1) else {
        return ExpectedSnafu {
          what: "Closing quote (\")",
          offset: reader.offset()
        }.fail();
      };

      match ch {
        consts::punctuation::QUOTE => { break; },
        consts::punctuation::BACKSLASH => {
          match reader.read_ch() {
            Ok('\'') => text.push('\''),
            Ok('"') => text.push('"'),
            Ok('\\') => text.push('\\'),
            Ok(consts::ascii::NL_ESCAPE) => text.push(consts::ascii::NL),
            Ok(consts::ascii::BL_ESCAPE) => text.push(consts::ascii::BL),
            Ok(consts::ascii::BS_ESCAPE) => text.push(consts::ascii::BS),
            Ok(consts::ascii::HT_ESCAPE) => text.push(consts::ascii::HT),
            Ok(consts::ascii::LF_ESCAPE) => text.push(consts::ascii::LF),
            Ok(consts::ascii::VT_ESCAPE) => text.push(consts::ascii::VT),
            Ok(consts::ascii::FF_ESCAPE) => text.push(consts::ascii::FF),
            Ok(consts::ascii::CR_ESCAPE) => text.push(consts::ascii::CR),
            Ok(consts::ascii::ES_ESCAPE) => text.push(consts::ascii::ES),
            Ok(consts::ascii::HEX_ESCAPE) => {
              return NotImplementedSnafu {
                what: "Hexadecimal Escape",
                offset: reader.offset()
              }.fail();
            },
            Ok(consts::ascii::UNI_ESCAPE) => {
              return NotImplementedSnafu {
                what: "Unicode Escape",
                offset: reader.offset()
              }.fail();
            },
            Ok(escaped) => {
              return UnknownSnafu {
                what: format!(
                  "Escaped Character ({:#?})",
                  escaped
                ),
                offset:
                reader.offset()
              }.fail();
            },
            Err(_) => {
              return ExpectedSnafu {
                what: "Escaped Character",
                offset: reader.offset()
              }.fail();
            }
          }
        },
        _ => { text.push_str(ch); }
      };
    };

    Ok(text)
  }
}

impl LiteralAST {
  pub fn make_numeric(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    loop {
      match reader.peek_ch() {
        Some('0'..='9') => {
          reader.seek(1).unwrap();
        },
        _ => { break; }
      }
    };

    if reader.offset() == start {
      return ExpectedSnafu {
        what: "Numeric Literal",
        offset: reader.offset()
      }.fail();
    }

    match true {
      _ if seek::begins_with(reader, consts::punctuation::U8_SFX) => {},
      _ if seek::begins_with(reader, consts::punctuation::U16_SFX) => {},
      _ if seek::begins_with(reader, consts::punctuation::U32_SFX) => {},
      _ if seek::begins_with(reader, consts::punctuation::U64_SFX) => {},
      _ if seek::begins_with(reader, consts::punctuation::U128_SFX) => {},
      _ if seek::begins_with(reader, consts::punctuation::USIZE_SFX) => {},
      _ if seek::begins_with(reader, consts::punctuation::I8_SFX) => {},
      _ if seek::begins_with(reader, consts::punctuation::I16_SFX) => {},
      _ if seek::begins_with(reader, consts::punctuation::I32_SFX) => {},
      _ if seek::begins_with(reader, consts::punctuation::I64_SFX) => {},
      _ if seek::begins_with(reader, consts::punctuation::I128_SFX) => {},
      _ if seek::begins_with(reader, consts::punctuation::ISIZE_SFX) => {},
      _ => {}
    };

    Ok(Self {
      span: reader.span_since(start),
      l: Literal::NumericLiteral(
        String::from(&reader.src()[start..reader.offset()])
      )
    })
  }

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if let Ok(l) = Literal::make(reader) {
      Ok(Self {
        span: reader.span_since(start), l
      })
    } else if let Some(l) = try_make!(LiteralAST::make_numeric, reader) {
      Ok(l)
    } else {
      ExpectedSnafu {
        what: "Literal",
        offset: reader.offset()
      }.fail()
    }
  }
}
