/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::{
  ast::*,
  SourceReader,
  AsterResult,
  seek_read::seek,
  errors::*
};

impl KeywordAST {
  pub fn make(reader: &mut SourceReader, text: &str) -> AsterResult<Self> {
    let start = reader.offset();

    if seek::begins_with(reader, text) {
      Ok(Self {
        span: reader.span_since(start)
      })
    } else {
      ExpectedSnafu {
        what: format!("Keyword ({})", text),
        offset: reader.offset()
      }.fail()
    }
  }
}
