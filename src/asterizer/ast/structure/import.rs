use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
};

use crate::tokenizer::{
  TokenEnum,
  Keyword,
};

use crate::compiler::Handle;

#[derive(Debug, TypeName)]
pub(crate) struct Qualify {
  pub(crate) name: String,
  pub(crate) rest: Box<ImportPattern>,
}

#[derive(Debug, TypeName)]
pub(crate) enum ImportPattern {
  Qualify(Qualify),
  Tail(String),
  Wildcard,
}

pub(crate) enum TopLevelImportPattern {
  All(String),
  Brace(ImportPattern),
}

#[derive(Debug, TypeName)]
pub(crate) struct Import {
  pub(crate) pattern: ImportPattern,
  pub(crate) from: Handle
}

impl MakeAst for Import {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::Import))

    todo!()
  }
}
