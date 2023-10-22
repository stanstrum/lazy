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
  errors::*,
  consts,
  seek
};

use crate::{
  try_make,
  intent
};

impl TraitAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let template = try_make!(TemplateAST::make, reader);

    if template.is_some() {
      seek::optional_whitespace(reader)?;
    };

    if !seek::begins_with(reader, consts::keyword::TRAIT) {
      return ExpectedSnafu {
        what: "Keyword (trait)",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    seek::required_whitespace(reader)?;

    let ident = intent!(IdentAST::make, reader)?;

    seek::optional_whitespace(reader)?;

    if !seek::begins_with(reader, consts::grouping::OPEN_BRACE) {
      return reader.set_intent(
        ExpectedSnafu {
          what: "Open Brace",
          offset: reader.offset(),
          path: reader.path.clone()
        }.fail()
      );
    };

    let mut decls: Vec<MemberFunctionDeclAST> = vec![];

    loop {
      seek::optional_whitespace(reader)?;

      if seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
        break;
      };

      let decl = intent!(MemberFunctionDeclAST::make, reader)?;
      decls.push(decl);

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::SEMICOLON) {
        return reader.set_intent(
          ExpectedSnafu {
            what: "Semicolon",
            offset: reader.offset(),
            path: reader.path.clone()
          }.fail()
        );
      };
    };

    Ok(Self {
      span: reader.span_since(start),
      ident, decls,
      template
    })
  }
}
