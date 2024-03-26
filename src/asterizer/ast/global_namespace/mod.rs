import_export! {
  top_level_structure
}

use std::collections::HashMap;
use typename::TypeName;

use crate::tokenizer::{
  TokenEnum,
  Punctuation
};

use crate::asterizer::ast::{
  MakeAst,
  TokenStream
};

use crate::asterizer::error::*;

#[allow(unused)]
#[derive(Debug, Default, TypeName)]
pub(crate) struct GlobalNamespace {
  // file: std::path::PathBuf,
  pub children: HashMap<String, TopLevelStructure>
}

impl MakeAst for GlobalNamespace {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let mut structures: Vec<TopLevelStructure> = vec![];

    stream.skip_whitespace_and_comments();

    while stream.remaining() > 0 {
      let Some(struc) = stream.make()? else {
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
