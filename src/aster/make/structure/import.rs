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

enum ImportPattern {
  Qualify {
    span: Span,
    ident: IdentAST,
    child: Box<ImportPattern>
  },
  Brace {
    span: Span,
    children: Vec<ImportPattern>
  },
  Ident {
    span: Span,
    ident: IdentAST, alias: Option<IdentAST>
  }
}

impl GetSpan for ImportPattern {
  fn span(&self) -> Span {
    match self {
      ImportPattern::Qualify { span, .. } => span.clone(),
      ImportPattern::Brace { span, .. } => span.clone(),
      ImportPattern::Ident { span, .. } => span.clone(),
    }
  }
}

impl ImportPattern {
  fn make_qualify(reader: &mut SourceReader) -> AsterResult<Self> {
    NotImplementedSnafu {
      what: "make_qualify",
      offset: reader.offset(),
      path: reader.path.clone()
    }.fail()
  }

  fn make_brace(reader: &mut SourceReader) -> AsterResult<Self> {
    NotImplementedSnafu {
      what: "make_pattern",
      offset: reader.offset(),
      path: reader.path.clone()
    }.fail()
  }

  fn make_ident(reader: &mut SourceReader) -> AsterResult<Self> {
    NotImplementedSnafu {
      what: "make_ident",
      offset: reader.offset(),
      path: reader.path.clone()
    }.fail()
  }

  fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    if let Some(qualify) = try_make!(ImportPattern::make_qualify, reader) {
      Ok(qualify)
    } else if let Some(brace) = try_make!(ImportPattern::make_brace, reader) {
      Ok(brace)
    } else if let Some(ident) = try_make!(ImportPattern::make_ident, reader) {
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

  fn to_map(&self, ns: NamespaceAST) -> HashMap<String, Structure> {
    todo!("import_pattern_to_map")
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

    let pattern = intent!(ImportPattern::make, reader)?;

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

    let global = intent!(asterize, reader)?;
    // swap the old reader back
    unsafe { std::ptr::swap(new_reader, reader); };

    let map = pattern.to_map(global);

    Ok(Self {
      span: reader.span_since(start),
      imported: map
    })
  }
}
