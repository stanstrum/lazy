/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

pub(crate) mod ast;
use std::collections::HashMap;

use ast::*;
pub use ast::Span;

pub mod intrinsics;

mod errors;
use errors::*;

mod source_reader;
pub use source_reader::*;

mod seek_read;
use seek_read::seek;

mod make;
pub mod consts;

pub fn asterize(reader: &mut SourceReader) -> AsterResult<NamespaceAST> {
  let span = Span { start: 0, end: 0 };
  let ident = IdentAST { span, text: "global".to_string() };

  let mut global = NamespaceAST {
    span: Span { start: 0, end: reader.len() },
    ident, map: HashMap::new()
  };

  for unique_ctr in 0.. {
    seek::optional_whitespace(reader)?;
    if reader.remaining() == 0 {
      break;
    };

    let structure = Structure::make(reader)?;
    if !seek::begins_with(reader, consts::punctuation::SEMICOLON) {
      return reader.set_intent_error(ExpectedSnafu {
        what: "Punctuation (\";\")",
        offset: reader.offset()
      }.fail());
    };

    let key = structure.to_hashable(unique_ctr);
    // todo: https://stackoverflow.com/a/28512504/6496600
    global.map.insert(key, structure);
  };

  Ok(global)
}
