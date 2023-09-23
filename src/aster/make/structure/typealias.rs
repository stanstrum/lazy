/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::{
  ast::*,
  AsterResult,
  SourceReader,
  consts,
  errors::*,
  seek
};

impl TypeAliasAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::TYPE) {
      return ExpectedSnafu {
        what: "Keyword (type)",
        offset: reader.offset()
      }.fail();
    };

    seek::required_whitespace(reader)?;

    let ident = IdentAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    if !seek::begins_with(reader, consts::punctuation::BOLLOCKS) {
      return ExpectedSnafu {
        what: "Punctuation (\":=\")",
        offset: reader.offset()
      }.fail();
    };

    seek::optional_whitespace(reader)?;

    let ty = TypeAST::make(reader)?;

    Ok(Self {
      span: reader.span_since(start),
      ident, ty
    })
  }
}
