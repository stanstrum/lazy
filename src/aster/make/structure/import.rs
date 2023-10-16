/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use crate::{
  aster::{
    ast::*,
    SourceReader,
    errors::*,
    consts,
    seek,
    asterize
  },
  try_make, intent
};

impl ImportPatternAST {
  fn make_qualify(reader: &mut SourceReader) -> AsterResult<Self> {
    NotImplementedSnafu {
      what: "make_qualify",
      offset: reader.offset(),
      path: reader.path.clone()
    }.fail()
  }

  fn make_brace(reader: &mut SourceReader) -> AsterResult<Self> {
    // if let Some(qualify) = try_make!(ImportPatternAST::make_qualify, reader) {
    //   Ok(qualify)
    // } else if let Some(brace) = try_make!(ImportPatternAST::make_brace, reader) {
    //   Ok(brace)
    // } else if let Some(ident) = try_make!(ImportPatternAST::make_ident, reader) {
    //   Ok(ident)
    // }

    NotImplementedSnafu {
      what: "make_pattern",
      offset: reader.offset(),
      path: reader.path.clone()
    }.fail()
  }

  fn make_ident(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let ident = IdentAST::make(reader)?;

    let alias = {
      let whitespace = seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, "as") {
        reader.rewind(whitespace).unwrap();

        None
      } else {
        intent!(seek::required_whitespace, reader)?;

        let alias = intent!(IdentAST::make, reader)?;
        Some(alias)
      }
    };

    Ok(Self::Ident {
      span: reader.span_since(start),
      ident, alias
    })
  }

  fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    if let Some(brace) = try_make!(ImportPatternAST::make_brace, reader) {
      Ok(brace)
    } else if let Some(ident) = try_make!(ImportPatternAST::make_ident, reader) {
      Ok(ident)
    } else {
      reader.set_intent(
        ExpectedSnafu {
          what: "an import pattern",
          offset: reader.offset(),
          path: reader.path.clone(),
        }.fail()
      )
    }
  }
}

impl ImportAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::keyword::IMPORT) {
      return ExpectedSnafu {
        what: "Keyword (\"import\")",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail();
    };

    if reader.peek_ch().is_some_and(|ch| ch != '{') {
      seek::required_whitespace(reader)?;
    };

    let pattern = intent!(ImportPatternAST::make, reader)?;

    reader.rewind(1).unwrap();

    if reader.read_ch().unwrap() != '{' {
      intent!(seek::required_whitespace, reader)?;
    };

    if !seek::begins_with(reader, consts::keyword::FROM) {
      return reader.set_intent(
        ExpectedSnafu {
          what: "Keyword (\"from\")",
          offset: reader.offset(),
          path: reader.path.clone()
        }.fail()
      );
    };

    seek::optional_whitespace(reader)?;

    let from_start = reader.offset() + 1;
    let from = intent!(LiteralAST::make, reader)?;
    let Literal::UnicodeString(from) = from.l else {
      return reader.set_intent(
        BadLiteralSnafu {
          expected: "a String",
          offset: from.span.start,
          path: reader.path.clone()
        }.fail()
      );
    };

    let parent = reader.path.parent()
      .expect("file does not have a parent directory?");

    let path = parent.join(from);
    println!("{:?}", path);

    let src = match std::fs::read_to_string(&path) {
      Ok(src) => src,
      Err(err) => {
        return reader.set_intent(
          ImportIOSnafu {
            error: err.to_string(),
            offset: from_start,
            path: reader.path.clone(),
          }.fail()
        );
      },
    };

    let new_reader = &mut SourceReader::new(path, &src);
    let ns = intent!(asterize, new_reader)?;

    Ok(Self {
      span: reader.span_since(start),
      ns, pattern
    })
  }

  pub fn populate_map(&mut self, map: &mut HashMap<String, Structure>) -> AsterResult<()> {
    todo!("populate map")
  }
}
