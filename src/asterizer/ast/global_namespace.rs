/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use typename::TypeName;

use crate::asterizer::error::ExpectedSnafu;
use crate::tokenizer::{
  TokenEnum,
  Punctuation
};

use crate::asterizer::{
  TokenStream,
  AsterizerError,
  MakeAst,
  ast::TopLevelStructure
};

#[allow(unused)]
#[derive(Debug, Default, TypeName)]
pub(crate) struct GlobalNamespace {
  // file: std::path::PathBuf,
  children: HashMap<String, TopLevelStructure>
}

impl MakeAst for GlobalNamespace {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let mut structures = vec![];

    stream.skip_whitespace_and_comments();

    while stream.remaining() > 0 {
      let Some(struc) = stream.make::<TopLevelStructure>()? else {
        return ExpectedSnafu {
          what: "a top-level structure",
        }.fail();
      };

      structures.push(struc);

      stream.skip_whitespace_and_comments();

      let Some(TokenEnum::Punctuation(Punctuation::Semicolon)) = stream.next_variant() else {
        return ExpectedSnafu {
          what: "a semicolon",
        }.fail();
      };

      stream.skip_whitespace_and_comments();
    };

    if structures.is_empty() {
      return ExpectedSnafu {
        what: "a top-level structure"
      }.fail();
    };

    let children = structures.into_iter()
      .map(
        |child| (child.name(), child)
      )
      .collect();

    Ok(Some(Self { children }))
  }
}
