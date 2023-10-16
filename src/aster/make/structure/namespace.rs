/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::{
  aster::{
    SourceReader,
    AsterResult,
    ast::*,
    consts,
    seek,
    errors::*,
  },
  try_make
};

use std::collections::HashMap;

impl NamespaceAST {
  pub fn new(ident: IdentAST, span: Span) -> Self {
    Self {
      span, ident,
      map: HashMap::new(),
      imports: Vec::new(),
    }
  }

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::NAMESPACE) {
      return ExpectedSnafu {
        what: "Namespace",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    seek::required_whitespace(reader)?;

    let ident = IdentAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    let mut map: HashMap<String, Structure> = HashMap::new();
    if !seek::begins_with(reader, consts::grouping::OPEN_BRACE) {
      return ExpectedSnafu {
        what: "Open Brace",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    loop {
      seek::optional_whitespace(reader)?;

      if seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
        break;
      };

      if let Some(_) = try_make!(ImportAST::make, reader) {
        todo!("error for importing in nested namespace");
      } else if let Some(structure) = try_make!(Structure::make, reader) {
        let key = structure.to_hashable();

        Self::insert_unique(&mut map, key, structure)?;
      };

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::SEMICOLON) {
        return ExpectedSnafu {
          what: "Punctuation (\";\")",
          offset: reader.offset(),
          path: reader.path.clone()
        }.fail();
      };
    };

    Ok(Self {
      span: reader.span_since(start),
      ident, map,
      imports: vec![],
    })
  }

  pub fn insert_unique(map: &mut HashMap<String, Structure>, key: String, value: Structure) -> AsterResult<()> {
    match map.get(&key) {
      Some(existing) => {
        let a = existing.span();
        let b = value.span();

        DuplicateIdentSnafu {
          text: key,
          a, b
        }.fail()
      },
      None => {
        map.insert(key, value);

        Ok(())
      },
    }
  }
}
