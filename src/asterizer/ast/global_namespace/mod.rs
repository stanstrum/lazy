import_export!(top_level_structure);

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
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
  pub(crate) children: Vec<TopLevelStructure>,
}

impl MakeAst for GlobalNamespace {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let mut structures: Vec<TopLevelStructure> = vec![];

    stream.skip_whitespace_and_comments();

    while stream.remaining() > 0 {
      let Some(structure) = stream.make()? else {
        return ExpectedSnafu {
          what: "a top-level structure",
          span: stream.span()
        }.fail();
      };

      structures.push(structure);
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

    Ok(Some(Self {
      children: structures,
    }))
  }
}
