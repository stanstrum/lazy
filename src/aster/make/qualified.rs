/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::{
  aster::{
    ast::*,
    SourceReader,
    AsterResult,
    seek_read::seek,
    consts,
    errors::*
  },
  try_make,
  intent
};

impl QualifiedAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();
    let mut parts: Vec<IdentAST> = vec![];

    let first = IdentAST::make(reader)?;
    parts.push(first);

    loop {
      let before_double_colon = reader.offset();

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::DOUBLE_COLON) {
        reader.to(before_double_colon).unwrap();

        break;
      };

      seek::optional_whitespace(reader)?;

      let Ok(part) = IdentAST::make(reader) else {
        reader.to(before_double_colon).unwrap();

        break;
      };

      parts.push(part);
    };

    Ok(Self {
      span: reader.span_since(start), parts
    })
  }
}
