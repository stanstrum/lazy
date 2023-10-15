/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use crate::aster::{
  ast::*,
  errors::*,
  consts,
  SourceReader,
  seek,
  intrinsics,
};

impl ExternDeclAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::EXTERN) {
      return ExpectedSnafu {
        what: "Keyword (\"extern\")",
        offset: reader.offset(),
      }.fail();
    };

    seek::required_whitespace(reader)?;

    let ident = IdentAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    let ret = {
      if seek::begins_with(reader, consts::punctuation::RIGHT_ARROW) {
        seek::optional_whitespace(reader)?;

        TypeAST::make(reader)?
      } else {
        TypeAST {
          span: ident.span(),
          e: Type::Intrinsic(intrinsics::VOID),
        }
      }
    };

    seek::optional_whitespace(reader)?;

    let mut varargs = false;

    let mut args: HashMap<IdentAST, TypeAST> = HashMap::new();
    if seek::begins_with(reader, consts::punctuation::COLON) {
      loop {
        seek::optional_whitespace(reader)?;

        if seek::begins_with(reader, consts::punctuation::ELLIPSIS) {
          varargs = true;

          break;
        };

        let arg_ty = TypeAST::make(reader)?;

        seek::required_whitespace(reader)?;

        let arg_ident = IdentAST::make(reader)?;

        seek::optional_whitespace(reader)?;

        args.insert(arg_ident, arg_ty);

        if !seek::begins_with(reader, consts::punctuation::COMMA) {
          break;
        };
      };
    };

    Ok(Self {
      span: reader.span_since(start),
      ident, ret,
      args,
      varargs
    })
  }
}
