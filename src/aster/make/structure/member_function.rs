/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::try_make;

use crate::aster::{
  ast::*,
  SourceReader,
  errors::*,
  consts,
  seek
};

impl MemberFunctionDeclAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let public = try_make!(KeywordAST::make, reader, consts::keyword::PUB);

    if public.is_some() {
      seek::required_whitespace(reader)?;
    };

    let r#static = try_make!(KeywordAST::make, reader, consts::keyword::STATIC);

    if r#static.is_some() {
      seek::required_whitespace(reader)?;
    };

    let r#mut = try_make!(KeywordAST::make, reader, consts::keyword::MUT);

    if r#mut.is_some() {
      seek::required_whitespace(reader)?;
    };

    let decl = FunctionDeclAST::make(reader)?;

    Ok(Self {
      span: reader.span_since(start),
      public, r#static, r#mut, decl
    })
  }
}

impl MemberFunctionAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let template = try_make!(TemplateAST::make, reader);

    if template.is_some() {
      seek::optional_whitespace(reader)?;
    };

    let decl = MemberFunctionDeclAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    let body = BlockExpressionAST::make(reader)?;

    Ok(Self {
      span: reader.span_since(start),
      decl, body,
      template
    })
  }
}
