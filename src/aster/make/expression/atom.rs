/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

 use super::super::{
  super::{
    ast::*,
    SourceReader,
    AsterResult,
    errors::*,
  },
  try_make,
};

impl AtomExpressionAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if let Some(lit) = try_make!(LiteralAST::make, reader) {
      Ok(Self {
        span: reader.span_since(start),
        a: AtomExpression::Literal(lit),
        out: Type::Unresolved,
      })
    } else if let Some(qual) = try_make!(QualifiedAST::make, reader) {
      Ok(Self {
        span: reader.span_since(start),
        out: Type::Unresolved,
        a: AtomExpression::Variable(qual)
      })
    } else {
      UnknownSnafu {
        what: "Expression",
        offset: reader.offset()
      }.fail()
    }
  }
}
