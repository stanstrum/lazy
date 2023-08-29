/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

 use crate::{aster::{
  ast::*,
  SourceReader,
  errors::*,
  seek_read::seek,
  consts
}, try_make};

impl ControlFlowAST {
  pub fn make_while(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::WHILE) {
      return ExpectedSnafu {
        what: "Keyword (while)",
        offset: reader.offset()
      }.fail();
    };

    seek::optional_whitespace(reader)?;

    let cond = Expression::make(reader)?;

    seek::optional_whitespace(reader)?;

    let body = BlockExpressionAST::make(reader)?;

    Ok(Self {
      span: reader.span_since(start),
      e: ControlFlow::While(
        Box::new(cond), Box::new(body)
      )
    })
  }

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    if let Some(r#while) = try_make!(ControlFlowAST::make_while, reader) {
      Ok(r#while)
    } else /* if let Some(r#if) = try_make!(ControlFlow) */ {
      ExpectedSnafu {
        what: "Control Flow",
        offset: reader.offset()
      }.fail()
    }
  }
}
