/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

pub(crate) mod ast;
use std::collections::HashMap;

use ast::*;
pub use ast::Span;

mod intrinsics;

mod errors;
use errors::*;

mod source_reader;
pub use source_reader::*;

mod seek_read;
use seek_read::{seek, read};

mod make;
mod consts;
mod to_string;

pub fn asterize(reader: &mut SourceReader) -> AsterResult<NamespaceAST> {
  let span = Span { start: 0, end: 0 };
  let ident = IdentAST { span, text: "global".to_string() };

  let mut global = NamespaceAST {
    span: Span { start: 0, end: reader.len() },
    ident, map: HashMap::new()
  };

  // println!("{}", reader.at());

  loop {
    seek::optional_whitespace(reader)?;

    if reader.remaining() == 0 {
      break;
    };

    if read::begins_with(reader, consts::keyword::FN) {
      let func = FunctionAST::make(reader)?;

      global.map.insert(
        func.ident.text.clone(),
        Structure::FunctionAST(func)
      );
    } else {
      return UnknownSnafu { what: "Structure", offset: reader.offset() }.fail();
    };
  };

  Ok(global)
}
