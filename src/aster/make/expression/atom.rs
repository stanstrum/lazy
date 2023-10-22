/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::{
  ast::*,
  SourceReader,
  AsterResult,
  errors::*,
  seek,
  consts
};

use crate::{
  try_make,
  intent
};

impl StructInitializerAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let fqual = FullyQualifiedAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    if !seek::begins_with(reader, consts::grouping::OPEN_BRACE) {
      return ExpectedSnafu {
        what: "Open Brace",
        offset: reader.offset(),
        path: reader.path.clone(),
      }.fail();
    };

    let mut members = Vec::<(IdentAST, Expression)>::new();
    loop {
      seek::optional_whitespace(reader)?;

      if seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
        break;
      };

      let ident = intent!(IdentAST::make, reader)?;

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::COLON) {
        return reader.set_intent(
          ExpectedSnafu {
            what: "Colon",
            offset: reader.offset(),
            path: reader.path.clone(),
          }.fail()
        );
      };

      seek::optional_whitespace(reader)?;

      let expr = intent!(Expression::make, reader)?;
      members.push((ident, expr));

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::COMMA) {
        if !seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
          return reader.set_intent(
            ExpectedSnafu {
              what: "Comma or Semicolon",
              offset: reader.offset(),
              path: reader.path.clone(),
            }.fail()
          );
        };

        break;
      };
    };

    Ok(Self {
      span: reader.span_since(start),
      fqual, members
    })
  }
}

impl AtomExpressionAST {
  fn make_return(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let is_ident = IdentAST::make(reader).is_ok();
    reader.to(start).unwrap();

    if is_ident || !seek::begins_with(reader, consts::keyword::RETURN) {
      return ExpectedSnafu {
        what: "Keyword (return)",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    let whitespace = seek::optional_whitespace(reader)?;

    let expr = 'expr: {
      if whitespace == 0 {
        break 'expr None;
      };

      if let Ok(expr) = Expression::make(reader) {
        Some(Box::new(expr))
      } else {
        reader.rewind(whitespace).unwrap();

        None
      }
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
    } else if let Some(initializer) = try_make!(StructInitializerAST::make, reader) {
      Ok(Self {
        span: reader.span_since(start),
        a: AtomExpression::StructInitializer(initializer),
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
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail()
    }
  }
}
