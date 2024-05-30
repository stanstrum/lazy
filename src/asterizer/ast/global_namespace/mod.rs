import_export!(top_level_structure);

use std::collections::HashMap;
use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  Import,
};

use crate::tokenizer::{
  TokenEnum,
  Punctuation,
};

use crate::asterizer::error::*;

#[allow(unused)]
#[derive(Debug, Default, TypeName)]
pub(crate) struct GlobalNamespace {
  // file: std::path::PathBuf,
  pub(crate) children: HashMap<String, TopLevelStructure>,
  pub(crate) imports: Vec<Import>,
}

impl MakeAst for GlobalNamespace {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let mut structures: Vec<TopLevelStructure> = vec![];
    let mut imports = vec![];

    stream.skip_whitespace_and_comments();

    while stream.remaining() > 0 {
      if let Some(import) = stream.make()? {
        imports.push(import);
      } else {
        let Some(structure) = stream.make()? else {
          return ExpectedSnafu {
            what: "a top-level structure",
            span: stream.span()
          }.fail();
        };

        structures.push(structure);
      };

      stream.skip_whitespace_and_comments();

      let Some(TokenEnum::Punctuation(Punctuation::Semicolon)) = stream.next_variant() else {
        return ExpectedSnafu {
          what: "a semicolon",
          span: stream.span()
        }.fail();
      };

      stream.skip_whitespace_and_comments();
    };

    if structures.is_empty() {
      return ExpectedSnafu {
        what: "a top-level structure",
        span: stream.span()
      }.fail();
    };

    let children = structures.into_iter()
      .map(
        |child| (child.name(), child)
      )
      .collect();

    Ok(Some(Self { children, imports }))
  }
}
