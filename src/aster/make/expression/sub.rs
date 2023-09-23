/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::super::super::{
  ast::*,
  SourceReader,
  AsterResult,
  consts,
  seek_read::seek,
  errors::*
};

impl SubExpressionAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::grouping::OPEN_PARENTHESIS) {
      return ExpectedSnafu {
        what: "Open Parenthesis",
        offset: reader.offset()
      }.fail();
    };

    seek::optional_whitespace(reader)?;

    let expr = Expression::make(reader)?;

    seek::optional_whitespace(reader)?;

    if !seek::begins_with(reader, consts::grouping::CLOSE_PARENTHESIS) {
      return ExpectedSnafu {
        what: "Close Parenthesis",
        offset: reader.offset()
      }.fail();
    };

    Ok(Self {
      span: reader.span_since(start),
      out: Type::Unresolved,
      e: Box::new(expr)
    })
  }
}
