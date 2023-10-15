/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use crate::aster::{
  ast::*,
  SourceReader,
  errors::*,
  consts,
  seek,
  asterize
};

enum ImportPattern {
  Qualify(IdentAST, Box<ImportPattern>),
  Brace(Vec<ImportPattern>),
  Ident(IdentAST, Option<String>)
}

fn import_pattern_to_map(pattern: ImportPattern, ns: NamespaceAST) -> HashMap<String, Structure> {
  todo!("import_pattern_to_map")
}

impl ImportAST {
  fn make_pattern(reader: &mut SourceReader) -> AsterResult<ImportPattern> {
    todo!("make_pattern")
  }

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::IMPORT) {
      return ExpectedSnafu {
        what: "Keyword (\"import\")",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    if reader.peek_ch().unwrap() != '{' {
      seek::required_whitespace(reader)?;
    };

    let pattern = Self::make_pattern(reader)?;

    reader.rewind(1).unwrap();

    if reader.read_ch().unwrap() != '{' {
      seek::required_whitespace(reader)?;
    };

    if !seek::begins_with(reader, consts::keyword::FROM) {
      return ExpectedSnafu {
        what: "Keyword (\"from\")",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    seek::optional_whitespace(reader)?;

    let from = LiteralAST::make(reader)?;
    let Literal::UnicodeString(from) = from.l else {
      return BadLiteralSnafu {
        expected: "a String",
        offset: from.span.start,
        path: reader.path.clone()
      }.fail();
    };

    let path = reader.path.join(from);
    let src = match std::fs::read_to_string(&path) {
      Ok(src) => src,
      Err(err) => {
        return ImportIOSnafu {
          path,
          error: err.to_string(),
          offset: reader.offset(),
        }.fail();
      },
    };

    let ref mut new_reader = unsafe {
      let reader = SourceReader::new(path, &src);

      // this will be okay since nowhere do we save a string slice
      // to the source. we do this because we want the AST to be
      // valid even if the reader is destroyed to save memory.
      // additionally, none of the parsing of the imported file
      // will require the source of the file that imported it.
      std::mem::transmute(reader)
    };
    // swap the new reader into place
    unsafe { std::ptr::swap(new_reader, reader); };

    let global = asterize(reader)?;
    // swap the old reader back
    unsafe { std::ptr::swap(new_reader, reader); };

    let map = import_pattern_to_map(pattern, global);

    Ok(Self {
      span: reader.span_since(start),
      imported: map
    })
  }
}
