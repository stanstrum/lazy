/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::{
  AsterResult,
  SourceReader,
  seek,
  errors::*,
  ast::*,
  consts
};

impl StructAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::STRUCT) {
      return ExpectedSnafu {
        what: "Struct",
        offset: reader.offset()
      }.fail();
    };

    seek::required_whitespace(reader)?;

    let ident = IdentAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    if !seek::begins_with(reader, consts::grouping::OPEN_BRACE) {
      return ExpectedSnafu {
        what: "Open Brace",
        offset: reader.offset()
      }.fail();
    };

    let mut members: Vec<(TypeAST, IdentAST)> = vec![];
    loop {
      seek::optional_whitespace(reader)?;

      if seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
        break;
      };

      let ty = TypeAST::make(reader)?;
      seek::required_whitespace(reader)?;

      let ident = IdentAST::make(reader)?;
      seek::optional_whitespace(reader)?;

      members.push((ty, ident));

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::COMMA) {
        if !seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
          return ExpectedSnafu {
            what: "Close Brace",
            offset: reader.offset()
          }.fail();
        };

        break;
      };
    };

    Ok(Self {
      span: reader.span_since(start),
      ident, members
    })
  }
}
