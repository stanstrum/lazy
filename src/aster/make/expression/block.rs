/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::super::super::{
  ast::*,
  SourceReader,
  AsterResult,
  seek_read::seek,
  consts,
  errors::*
};

impl BlockExpressionAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::grouping::OPEN_BRACE) {
      return ExpectedSnafu {
        what: "Open Curly Brace",
        offset: reader.offset()
      }.fail();
    };

    let mut children: Vec<Expression> = vec![];

    let returns_last = loop {
      seek::optional_whitespace(reader)?;

      if seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
        break false;
      };

      if let Ok(expr) = AtomExpressionAST::make(reader) {
        children.push(Expression::Atom(expr));
      } else if let Ok(expr) = BlockExpressionAST::make(reader) {
        children.push(Expression::Block(expr));
      } else {
        return ExpectedSnafu {
          what: "Expression (block, atom)",
          offset: reader.offset()
        }.fail();
      };

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::SEMICOLON) {
        if seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
          break true;
        } else {
          return ExpectedSnafu {
            what: "Close Curly Brace or Semicolon",
            offset: reader.offset()
          }.fail();
        };
      };
    };

    Ok(Self {
      span: reader.span_since(start),
      children, returns_last, out: Type::Unresolved
    })
  }
}
