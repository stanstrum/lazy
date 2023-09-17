/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::{
  ast::*,
  AsterResult,
  SourceReader,
  errors::*,
  consts,
  seek_read::seek
};

impl TraitAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::TRAIT) {
      return ExpectedSnafu {
        what: "Keyword (trait)",
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

    let mut decls: Vec<MemberFunctionDeclAST> = vec![];

    loop {
      seek::optional_whitespace(reader)?;

      if seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
        break;
      };

      decls.push(MemberFunctionDeclAST::make(reader)?);

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::SEMICOLON) {
        return ExpectedSnafu {
          what: "Punctuation (\";\")",
          offset: reader.offset()
        }.fail();
      };
    };

    Ok(Self {
      span: reader.span_since(start),
      ident,
      decls,
    })
  }
}
