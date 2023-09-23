/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::source_reader::SourceReader;
use super::errors::*;

use super::consts;

pub mod seek {
  use super::*;

  fn line_comment(reader: &mut SourceReader) -> AsterResult<()> {
    if !begins_with(reader, consts::punctuation::LINE_COMMENT) {
      return ExpectedSnafu { what: "Line Comment (//)", offset: reader.offset() }.fail();
    };

    loop {
      match reader.read_ch() {
        Ok('\n') | Err(_) => { break; },
        _ => ()
      };
    };

    Ok(())
  }

  fn multiline_comment(reader: &mut SourceReader) -> AsterResult<()> {
    if !begins_with(reader, consts::grouping::OPEN_MULTILINE_COMMENT) {
      return ExpectedSnafu { what: "Open Multiline Comment (/*)", offset: reader.offset() }.fail();
    };

    loop {
      if reader.remaining() == 0 {
        return ExpectedSnafu { what: "End Multiline Comment (*/)", offset: reader.offset() }.fail();
      };

      if begins_with(reader, consts::grouping::CLOSE_MULTILINE_COMMENT) {
        break;
      }

      reader.seek(1).unwrap();
    }

    Ok(())
  }

  pub fn optional_whitespace(reader: &mut SourceReader) -> AsterResult<usize> {
    let mut ctr: usize = 0;

    loop {
      if read::begins_with(reader, consts::punctuation::LINE_COMMENT) {
        line_comment(reader)?;
      } else if read::begins_with(reader, consts::grouping::OPEN_MULTILINE_COMMENT) {
        multiline_comment(reader)?;
      };

      match reader.read_ch() {
        Ok(' ' | '\r' | '\n' | '\t' | '\x0b') => (),
        Err(_) => { break; },
        _ => {
          reader.rewind(1).unwrap();

          break;
        }
      };

      ctr += 1;
    };

    Ok(ctr)
  }

  pub fn required_whitespace(reader: &mut SourceReader) -> AsterResult<usize> {
    let len = optional_whitespace(reader)?;

    if len != 0 {
      Ok(len)
    } else {
      ExpectedSnafu { what: "Whitespace", offset: reader.offset() }.fail()
    }
  }

  pub fn begins_with(reader: &mut SourceReader, a: &str) -> bool {
    let matches_input = read::begins_with(reader, a);

    if matches_input {
      reader.seek(a.len()).unwrap();
    };

    matches_input
  }
}

pub mod read {
  use super::SourceReader;

  pub fn begins_with(reader: &mut SourceReader, a: &str) -> bool {
    let Some(b) = reader.peek(a.len()) else {
      return false;
    };

    a == b
  }
}
