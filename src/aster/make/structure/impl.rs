/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::{
  ast::*,
  SourceReader, errors::*,
  consts,
  seek_read::seek
};

fn parse_methods(reader: &mut SourceReader) -> AsterResult<Vec<MemberFunctionAST>> {
  if !seek::begins_with(reader, consts::grouping::OPEN_BRACE) {
    return ExpectedSnafu {
      what: "Punctuation (\"{\")",
      offset: reader.offset(),
        path: reader.path.clone()
    }.fail();
  };

  let mut methods: Vec<MemberFunctionAST> = vec![];

  loop {
    seek::optional_whitespace(reader)?;

    if seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
      break;
    };

    methods.push(MemberFunctionAST::make(reader)?);

    if !seek::begins_with(reader, consts::punctuation::SEMICOLON) {
      return ExpectedSnafu {
        what: "Punctuation (\";\")",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };
  };

  Ok(methods)
}

impl ImplAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::IMPL) {
      return ExpectedSnafu {
        what: "Keyword (impl)",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    seek::required_whitespace(reader)?;

    let ty = TypeAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    let methods = parse_methods(reader)?;

    Ok(Self {
      span: reader.span_since(start),
      ty, methods
    })
  }
}

impl ImplForAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::IMPL) {
      return ExpectedSnafu {
        what: "Keyword (impl)",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    seek::required_whitespace(reader)?;

    let ty = TypeAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    if !seek::begins_with(reader, consts::punctuation::COLON) {
      return ExpectedSnafu {
        what: "Punctuation (\":\")",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    seek::optional_whitespace(reader)?;

    let r#trait = QualifiedAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    let methods = parse_methods(reader)?;

    Ok(Self {
      span: reader.span_since(start),
      r#trait, ty, methods,
    })
  }
}
