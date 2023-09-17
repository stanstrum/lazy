/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::{
  SourceReader,
  AsterResult,
  ast::*,
  consts,
  seek,
  errors::*,
};

use std::collections::HashMap;

impl NamespaceAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::NAMESPACE) {
      return ExpectedSnafu {
        what: "Namespace",
        offset: reader.offset()
      }.fail();
    };

    seek::required_whitespace(reader)?;

    let ident = IdentAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    let mut map: HashMap<String, Structure> = HashMap::new();
    if !seek::begins_with(reader, consts::grouping::OPEN_BRACE) {
      return ExpectedSnafu {
        what: "Open Brace",
        offset: reader.offset()
      }.fail();
    };

    loop {
      seek::optional_whitespace(reader)?;

      if seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
        break;
      };

      let (key, structure) = Structure::make(reader)?;
      map.insert(key, structure);

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::SEMICOLON) {
        return ExpectedSnafu {
          what: "Punctuation (\";\")",
          offset: reader.offset()
        }.fail();
      };
    };

    Ok(Self {
      span: reader.span_since(start),
      ident, map,
    })
  }
}
