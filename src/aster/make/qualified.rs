/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

 use super::{
  super::{
    ast::*,
    SourceReader,
    AsterResult,
    seek_read::seek,
    consts,
    errors::*
  },
  try_make
};

impl QualifiedAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();
    let mut parts: Vec<IdentAST> = vec![];

    loop {
      let Some(ident) = try_make!(IdentAST::make, reader) else {
        break;
      };

      parts.push(ident);

      let whitespace_len = seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::DOUBLE_COLON) {
        reader.rewind(whitespace_len).unwrap();

        break;
      };

      seek::optional_whitespace(reader)?;
    };

    if parts.is_empty() {
      return ExpectedSnafu {
        what: "Qualified Ident",
        offset: reader.offset()
      }.fail();
    };

    Ok(Self {
      span: reader.span_since(start), parts
    })
  }
}
