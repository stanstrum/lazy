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
  seek_read::seek,
  consts,
  errors::*,
};

use super::try_make;

use std::collections::HashMap;

impl BlockExpressionAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::grouping::OPEN_BRACE) {
      return ExpectedSnafu {
        what: "Open Curly Brace",
        offset: reader.offset()
      }.fail();
    };

    let mut children: Vec<BlockExpressionChild> = vec![];

    let returns_last = loop {
      seek::optional_whitespace(reader)?;

      if seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
        break false;
      };

      let child = {
        if let Some(binding) = try_make!(BindingAST::make, reader) {
          BlockExpressionChild::Binding(binding)
        } else if let Some(expr) = try_make!(Expression::make, reader) {
          BlockExpressionChild::Expression(expr)
        } else {
          return ExpectedSnafu {
            what: "Expression or Binding",
            offset: reader.offset()
          }.fail();
        }
      };

      children.push(child);

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
      children, returns_last,
      vars: HashMap::new(),
      out: Type::Unresolved
    })
  }
}
