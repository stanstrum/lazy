/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

pub(crate) mod ast;

use ast::*;
pub use ast::Span;

pub mod intrinsics;

mod errors;
use errors::*;

mod source_reader;
pub use source_reader::*;

mod seek_read;
use seek_read::seek;

use crate::try_make;

mod make;
pub mod consts;

pub fn asterize(reader: &mut SourceReader) -> AsterResult<NamespaceAST> {
  let path = reader.path.clone();

  let span = Span { start: 0, end: 0, path: path.clone() };
  let ident = IdentAST { span, text: "global".to_string() };

  let mut global = NamespaceAST::new(
    ident,
    Span {
      start: 0,
      end: reader.len(),
      path: path.clone()
    }
  );

  let map = &mut global.map;
  loop {
    seek::optional_whitespace(reader)?;
    if reader.remaining() == 0 {
      break;
    };

    if let Some(mut import) = try_make!(ImportAST::make, reader) {
      import.populate_map(map)?;
      global.imports.push(import);
    } else if let Some(structure) = try_make!(Structure::make, reader) {
      let key = structure.to_hashable();

      NamespaceAST::insert_unique(map, key, structure)?;
    } else {
      return ExpectedSnafu {
        what: "Structure",
        offset: reader.offset(),
        path: path.clone(),
      }.fail();
    };

    if !seek::begins_with(reader, consts::punctuation::SEMICOLON) {
      return reader.set_intent(
        ExpectedSnafu {
          what: "Punctuation (\";\")",
          offset: reader.offset(),
          path: reader.path.clone()
        }.fail()
      );
    };
  };

  Ok(global)
}
