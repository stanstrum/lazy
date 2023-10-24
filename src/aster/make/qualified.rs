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
  seek,
  consts,
  errors::*,
};

use crate::try_make;

impl QualifiedAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();
    let mut parts: Vec<IdentAST> = vec![];

    let first = IdentAST::make(reader)?;
    parts.push(first);

    loop {
      let before_double_colon = reader.offset();

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::DOUBLE_COLON) {
        reader.to(before_double_colon).unwrap();

        break;
      };

      seek::optional_whitespace(reader)?;

      let Ok(part) = IdentAST::make(reader) else {
        reader.to(before_double_colon).unwrap();

        break;
      };

      parts.push(part);
    };

    Ok(Self {
      span: reader.span_since(start), parts
    })
  }
}

impl FullyQualifiedIdentAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let ident = IdentAST::make(reader)?;

    let whitespace = seek::optional_whitespace(reader)?;

    if !seek::begins_with(reader, consts::grouping::OPEN_CHEVRON) {
      reader.rewind(whitespace).unwrap();

      return Ok(Self {
        span: reader.span_since(start),
        ident,
        generics: None,
      });
    };

    let mut generics = Vec::<TypeAST>::new();
    loop {
      seek::optional_whitespace(reader)?;

      if seek::begins_with(reader, consts::grouping::CLOSE_CHEVRON) {
        break;
      };

      let generic = TypeAST::make(reader)?;
      generics.push(generic);

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::COMMA) {
        seek::optional_whitespace(reader)?;

        if !seek::begins_with(reader, consts::grouping::CLOSE_CHEVRON) {
          return ExpectedSnafu {
            what: "Comma or Close Chevron",
            offset: reader.offset(),
            path: reader.path.clone(),
          }.fail();
        };

        break;
      };
    };

    Ok(Self {
      span: reader.span_since(start),
      ident,
      generics: Some(generics)
    })
  }

  pub fn to_hashable(&self) -> String {
    let mut text = self.ident.to_hashable();

    if let Some(generics) = &self.generics {
      text += "<";

      text += generics.iter()
        .map(|generic| generic.to_hashable())
        .collect::<Vec<_>>()
        .join(", ")
        .as_str();

      text += ">";
    };

    text
  }
}

impl FullyQualifiedAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let fqual = FullyQualifiedIdentAST::make(reader)?;

    let mut parts = vec![fqual];

    loop {
      let iter_start = reader.offset();

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::DOUBLE_COLON) {
        reader.to(iter_start).unwrap();

        break;
      };

      seek::optional_whitespace(reader)?;

      let Some(fqual) = try_make!(FullyQualifiedIdentAST::make, reader) else {
        reader.to(iter_start).unwrap();

        break;
      };

      parts.push(fqual);
    };

    Ok(Self {
      span: reader.span_since(start),
      parts
    })
  }

  pub fn to_hashable(&self) -> String {
    self.parts
      .iter()
      .map(|part| part.to_hashable())
      .collect::<Vec<_>>()
      .join("::")
  }
}

impl From<&IdentAST> for FullyQualifiedIdentAST {
  fn from(value: &IdentAST) -> Self {
    Self {
      span: value.span(),
      ident: value.to_owned(),
      generics: None,
    }
  }
}

impl From<&IdentAST> for FullyQualifiedAST {
  fn from(value: &IdentAST) -> Self {
    Self {
      span: value.span(),
      parts: vec![value.into()],
    }
  }
}

impl From<&QualifiedAST> for FullyQualifiedAST {
  fn from(value: &QualifiedAST) -> Self {
    let parts = value.parts
      .iter()
      .map(|part| part.into())
      .collect::<Vec<_>>();

    Self {
      span: value.span(),
      parts
    }
  }
}
