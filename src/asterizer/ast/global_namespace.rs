use std::collections::HashMap;

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

#[derive(Debug, Default)]
pub(crate) struct GlobalNamespace {
  // file: std::path::PathBuf,
  children: HashMap<String, TopLevelStructure>
}

impl MakeAst for GlobalNamespace {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    stream.push_mark();

    let mut structures = vec![];

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

    let children = structures.into_iter()
      .map(
        |child| (child.name(), child)
      )
      .collect();

    stream.drop_mark();

    Ok(Some(Self { children }))
  }
}
