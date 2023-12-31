/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use crate::aster::{
  SourceReader,
  AsterResult,
  ast::*,
  consts,
  seek_read::seek,
  intrinsics
};

use crate::{intent, try_make};

impl FunctionDeclAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let ident = IdentAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    // return type (optional)
    let ret = if seek::begins_with(reader, consts::punctuation::RIGHT_ARROW) {
      seek::optional_whitespace(reader)?;

      let ret = intent!(TypeAST::make, reader)?;

      seek::optional_whitespace(reader)?;

      ret
    } else {
      // sponge: this will show a type error with a void return type in a nontrivial place
      // due to the fact that a void return type is implicitly inferred
      TypeAST {
        span: reader.span_since(start),
        e: Type::Intrinsic(intrinsics::VOID)
      }
    };

    let mut args: HashMap<IdentAST, TypeAST> = HashMap::new();

    // arguments (optional)
    if seek::begins_with(reader, consts::punctuation::COLON) {
      loop {
        seek::optional_whitespace(reader)?;

        let arg_ty = intent!(TypeAST::make, reader)?;

        seek::required_whitespace(reader)?;

        let arg_ident = intent!(IdentAST::make, reader)?;

        seek::optional_whitespace(reader)?;

        args.insert(arg_ident, arg_ty);

        if !seek::begins_with(reader, consts::punctuation::COMMA) {
          break;
        };
      };

      seek::optional_whitespace(reader)?;
    };

    Ok(Self {
      span: reader.span_since(start),
      ident, args, ret
    })
  }
}

impl FunctionAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let template = try_make!(TemplateAST::make, reader);

    if template.is_some() {
      seek::optional_whitespace(reader)?;
    };

    let decl = FunctionDeclAST::make(reader)?;
    let body = intent!(BlockExpressionAST::make, reader)?;

    Ok(Self {
      span: reader.span_since(start),
      decl, body,
      template
    })
  }
}
