/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::{
  AsterResult,
  SourceReader,
  seek,
  consts,
  errors::*,
  ast::*,
  intrinsics
};

use super::*;
use crate::intent;

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
        // I'm not sure that any expression can start with a closing bracket
        // We shall find out
        return reader.set_intent(
          ExpectedSnafu {
            what: "Closing Bracket",
            offset: reader.offset(),
            path: reader.path.clone()
          }.fail()
        );
      };

      seek::optional_whitespace(reader)?;

      let ty = intent!(TypeAST::make, reader)?;
      let ty = Box::new(ty);

      Ok(Self {
        span: reader.span_since(start),
        e: Type::ArrayOf(len, ty)
      })
    } else if let Some(fqual) = try_make!(FullyQualifiedAST::make, reader) {
      let ty = match intrinsics::get_intrinsic(&fqual) {
        Some(ty) => ty,
        None => Type::Unknown(fqual),
      };

      Ok(Self {
        span: reader.span_since(start),
        e: ty
      })
    } else {
      return ExpectedSnafu {
        what: "Type",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    }
  }
}
