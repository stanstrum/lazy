/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

 use crate::aster::{
  AsterResult,

  SourceReader,
  seek_read::seek,

  consts,
  errors::*,
  ast::*,
  intrinsics
};

use super::*;

impl TypeAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if seek::begins_with(reader, consts::punctuation::AMPERSAND) {
      seek::optional_whitespace(reader)?;

      let ty = Box::new(TypeAST::make(reader)?);

      Ok(Self {
        span: reader.span_since(start),
        e: Type::ConstReferenceTo(ty)
      })
    } else if seek::begins_with(reader, consts::grouping::OPEN_BRACKET) {
      seek::optional_whitespace(reader)?;

      let len = try_make!(LiteralAST::make_numeric, reader);

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::grouping::CLOSE_BRACKET) {
        return ExpectedSnafu {
          what: "Closing Bracket",
          offset: reader.offset()
        }.fail();
      };

      seek::optional_whitespace(reader)?;

      let ty = Box::new(TypeAST::make(reader)?);

      Ok(Self {
        span: reader.span_since(start),
        e: Type::ArrayOf(len, ty)
      })
    } else {
      let Some(qual) = try_make!(QualifiedAST::make, reader) else {
        return ExpectedSnafu { what: "Qualified Ident", offset: reader.offset() }.fail();
      };

      Ok(Self {
        span: reader.span_since(start), e: match intrinsics::get_intrinsic(&qual) {
          Some(ty) => ty,
          None => Type::Unknown(qual),
        }
      })
    }
  }
}
