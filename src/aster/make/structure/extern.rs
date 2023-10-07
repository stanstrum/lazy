/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::{
  ast::*,
  errors::*,
  consts,
  SourceReader,
  seek,
};

impl ExternDeclAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::EXTERN) {
      return ExpectedSnafu {
        what: "Keyword (\"extern\")",
        offset: reader.offset(),
      }.fail();
    };

    seek::required_whitespace(reader)?;

    let decl = FunctionDeclAST::make(reader)?;

    Ok(Self {
      span: reader.span_since(start),
      decl,
    })
  }
}
