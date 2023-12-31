/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::{
  aster::{
    ast::*,
    SourceReader,
    errors::*,
    seek,
    consts
  },
  try_make,
  intent
};

impl ControlFlowAST {
  fn make_while(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::WHILE) {
      return ExpectedSnafu {
        what: "Keyword (while)",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    seek::required_whitespace(reader)?;

    let cond = intent!(Expression::make, reader)?;

    seek::required_whitespace(reader)?;

    let body = intent!(BlockExpressionAST::make, reader)?;

    Ok(Self {
      span: reader.span_since(start),
      e: ControlFlow::While(
        Box::new(cond), Box::new(body)
      )
    })
  }

  fn make_loop(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::LOOP) {
      return ExpectedSnafu {
        what: "Keyword (loop)",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    seek::optional_whitespace(reader)?;

    let expr = intent!(BlockExpressionAST::make, reader)?;
    let expr = Box::new(expr);

    Ok(Self {
      span: reader.span_since(start),
      e: ControlFlow::Loop(expr)
    })
  }

  fn make_if(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::IF) {
      return ExpectedSnafu {
        what: "Keyword (\"if\")",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    seek::required_whitespace(reader)?;

    let cond = intent!(Expression::make, reader)?;
    seek::required_whitespace(reader)?;

    let body = intent!(BlockExpressionAST::make, reader)?;

    let mut branches = vec![(cond, body)];

    let r#else = 'r_else: loop {
      let space_len = seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::keyword::ELSE) {
        reader.rewind(space_len).unwrap();

        break 'r_else None;
      };

      seek::required_whitespace(reader)?;

      if !seek::begins_with(reader, consts::keyword::IF) {
        let block = intent!(BlockExpressionAST::make, reader)?;
        break 'r_else Some(block);
      };

      seek::required_whitespace(reader)?;

      let cond = intent!(Expression::make, reader)?;
      seek::required_whitespace(reader)?;

      let body = intent!(BlockExpressionAST::make ,reader)?;

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
    } else if let Some(r#loop) = try_make!(ControlFlowAST::make_loop, reader) {
      Ok(r#loop)
    } else if let Some(r#if) = try_make!(ControlFlowAST::make_if, reader) {
      Ok(r#if)
    } else {
      ExpectedSnafu {
        what: "Control Flow",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail()
    }
  }
}
