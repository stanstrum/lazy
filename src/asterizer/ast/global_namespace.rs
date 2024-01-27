/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

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
#[derive(Debug, Default)]
pub(crate) struct GlobalNamespace {
  // file: std::path::PathBuf,
  children: HashMap<String, TopLevelStructure>
}

impl MakeAst for GlobalNamespace {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    println!("GlobalNamespace::make");

    stream.push_mark();

    let mut structures = vec![];

    stream.skip_whitespace_and_comments();

    while let Some(struc) = TopLevelStructure::make(stream)? {
      structures.push(struc);

      stream.skip_whitespace_and_comments();

      match stream.next_variant() {
        Some(TokenEnum::Punctuation(Punctuation::Semicolon)) => {},
        _ => {
          stream.pop_mark();

          // sponge: error here.
          eprintln!("expected semicolon");
          return Ok(None);
        },
      };
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

    stream.drop_mark();

    Ok(Some(Self { children }))
  }
}
