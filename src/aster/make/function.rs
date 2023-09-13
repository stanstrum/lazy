/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

 use super::super::{
  SourceReader,
  AsterResult,
  ast::*,
  consts,
  seek_read::seek,
  intrinsics
};

impl FunctionDeclAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let ident = IdentAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    // return type (optional)
    let ret = if seek::begins_with(reader, consts::punctuation::RIGHT_ARROW) {
      seek::optional_whitespace(reader)?;

      let ret = TypeAST::make(reader)?;

      seek::optional_whitespace(reader)?;

      ret
    } else {
      // sponge: this will show a type error with a void return type in a nontrivial place
      // due to the fact that a void return type is implicitly inferred
      TypeAST {
        span: reader.span_since(start),
        e: Type::Intrinsic(&intrinsics::VOID)
      }
    };

    let mut args: Vec<Variable> = vec![];

    // arguments (optional)
    if seek::begins_with(reader, consts::punctuation::COLON) {
      loop {
        seek::optional_whitespace(reader)?;

        let arg_ty = TypeAST::make(reader)?;

        seek::required_whitespace(reader)?;

        let arg_ident = IdentAST::make(reader)?;

        seek::optional_whitespace(reader)?;

        args.push(Variable(arg_ty, arg_ident));

        if !seek::begins_with(reader, consts::punctuation::COMMA) {
          break;
        }
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

    let decl = FunctionDeclAST::make(reader)?;

    let body = BlockExpressionAST::make(reader)?;

    Ok(Self {
      span: reader.span_since(start),
      decl, body
    })
  }
}
