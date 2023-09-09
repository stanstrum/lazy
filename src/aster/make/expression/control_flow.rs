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
  fn make_while(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::WHILE) {
      return ExpectedSnafu {
        what: "Keyword (while)",
        offset: reader.offset()
      }.fail();
    };

    seek::required_whitespace(reader)?;

    let cond = Expression::make(reader)?;

    seek::required_whitespace(reader)?;

    let body = BlockExpressionAST::make(reader)?;

    Ok(Self {
      span: reader.span_since(start),
      e: ControlFlow::While(
        Box::new(cond), Box::new(body)
      )
    })
  }

  fn make_if(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::IF) {
      return ExpectedSnafu {
        what: "Keyword (if)",
        offset: reader.offset()
      }.fail();
    };

    seek::required_whitespace(reader)?;

    dbg!(reader.offset());
    let cond = Expression::make(reader)?;
    dbg!(reader.offset());
    seek::required_whitespace(reader)?;

    let body = BlockExpressionAST::make(reader)?;

    let mut branches = vec![(cond, body)];

    let r#else = 'r_else: loop {
      let space_len = seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::keyword::ELSE) {
        reader.rewind(space_len).unwrap();

        break 'r_else None;
      };

      seek::required_whitespace(reader)?;

      if !seek::begins_with(reader, consts::keyword::IF) {
        break 'r_else Some(BlockExpressionAST::make(reader)?);
      };

      seek::required_whitespace(reader)?;

      let cond = Expression::make(reader)?;
      seek::required_whitespace(reader)?;

      let body = BlockExpressionAST::make(reader)?;

      branches.push((cond, body));
    };

    Ok(Self {
      span: reader.span_since(start),
      e: ControlFlow::If(branches, r#else)
    })
  }

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    if let Some(r#while) = try_make!(ControlFlowAST::make_while, reader) {
      Ok(r#while)
    } else if let Some(r#if) = try_make!(ControlFlowAST::make_if, reader) {
      Ok(r#if)
    } else {
      ExpectedSnafu {
        what: "Control Flow",
        offset: reader.offset()
      }.fail()
    }
  }
}
