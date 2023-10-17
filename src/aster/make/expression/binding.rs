/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::{
  ast::*,
  errors::*,
  consts,
  seek,
  SourceReader,
  AsterResult,
};

use super::{
  try_make,
  intent
};

#[derive(Debug)]
struct TypeIdentPair(TypeAST, IdentAST);

impl GetSpan for TypeIdentPair {
  fn span(&self) -> Span {
    let (ty, ident) = (&self.0, &self.1);

    Span {
      start: ty.span.start,
      end: ident.span.end,
      path: ident.span.path.clone()
    }
  }
}

impl BindingAST {
  fn make_type_ident(reader: &mut SourceReader) -> AsterResult<TypeIdentPair> {
    let ty = TypeAST::make(reader)?;

    seek::required_whitespace(reader)?;

    let ident = IdentAST::make(reader)?;

    Ok(TypeIdentPair(ty, ident))
  }

  fn make_value(reader: &mut SourceReader) -> AsterResult<Expression> {
    seek::optional_whitespace(reader)?;

    if !seek::begins_with(reader, consts::punctuation::BOLLOCKS) {
      return ExpectedSnafu {
        what: "Punctuation (\":=\")",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    seek::optional_whitespace(reader)?;

    intent!(Expression::make, reader)
  }

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let r#mut = try_make!(KeywordAST::make, reader, consts::keyword::MUT);

    if r#mut.is_some() {
      seek::required_whitespace(reader)?;
    };

    let (ty, ident) = if
      let Some(
        TypeIdentPair(ty, ident)
      ) = try_make!(BindingAST::make_type_ident, reader)
    {
      (Some(ty), ident)
    } else {
      let ident = IdentAST::make(reader)?;

      (None, ident)
    };

    let value = try_make!(BindingAST::make_value, reader)
      .map(Box::new);

    if ty.is_none() && value.is_none() {
      return ExpectedSnafu {
        what: "Value expression",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    Ok(Self {
      span: reader.span_since(start),
      r#mut, ty, ident, value,
    })
  }
}
