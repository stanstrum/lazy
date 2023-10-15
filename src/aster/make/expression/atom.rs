/* Copyright (c) 2023 Stan Strum
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
    seek,
    consts
  },
  try_make,
};

impl AtomExpressionAST {
  fn make_return(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let is_ident = IdentAST::make(reader).is_ok();
    reader.to(start).unwrap();

    if is_ident || !seek::begins_with(reader, consts::keyword::RETURN) {
      return ExpectedSnafu {
        what: "Keyword (return)",
        offset: reader.offset()
      }.fail();
    };

    let after_ret = reader.offset();

    let expr = if seek::optional_whitespace(reader)? != 0 {
      if let Ok(expr) = Expression::make(reader) {
        Some(Box::new(expr))
      } else {
        reader.to(after_ret).unwrap();

        None
      }
    } else {
      None
    };

    Ok(Self {
      span: reader.span_since(start),
      out: Type::Unresolved,
      a: AtomExpression::Return(expr),
    })
  }

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if let Some(ret) = try_make!(AtomExpressionAST::make_return, reader) {
      Ok(ret)
    } else if let Some(lit) = try_make!(LiteralAST::make, reader) {
      Ok(Self {
        span: reader.span_since(start),
        a: AtomExpression::Literal(lit),
        out: Type::Unresolved,
      })
    } else if let Some(qual) = try_make!(QualifiedAST::make, reader) {
      Ok(Self {
        span: reader.span_since(start),
        out: Type::Unresolved,
        a: AtomExpression::UnresolvedVariable(qual)
      })
    } else {
      UnknownSnafu {
        what: "Expression",
        offset: reader.offset()
      }.fail()
    }
  }
}
