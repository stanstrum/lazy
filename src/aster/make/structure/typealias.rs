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

use crate::{
  try_make,
  intent
};

impl TypeAliasAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let template = try_make!(TemplateAST::make, reader);

    if template.is_some() {
      seek::optional_whitespace(reader)?;
    };

    if !seek::begins_with(reader, consts::keyword::TYPE) {
      return ExpectedSnafu {
        what: "Keyword (type)",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    seek::required_whitespace(reader)?;

    let ident = IdentAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    if !seek::begins_with(reader, consts::punctuation::BOLLOCKS) {
      return reader.set_intent(
        ExpectedSnafu {
          what: "Punctuation (\":=\")",
          offset: reader.offset(),
          path: reader.path.clone()
        }.fail()
      );
    };

    seek::optional_whitespace(reader)?;

    let ty = intent!(TypeAST::make, reader)?;

    Ok(Self {
      span: reader.span_since(start),
      ident, ty,
      template
    })
  }
}
