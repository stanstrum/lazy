use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  FunctionDeclaration,
};

use crate::tokenizer::{
  TokenEnum,
  Keyword,
};

use crate::asterizer::error::ExpectedSnafu;

#[derive(Debug, TypeName)]
pub(crate) struct Extern {
  pub(crate) decl: FunctionDeclaration
}

impl MakeAst for Extern {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::Extern)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(decl) = stream.make()? else {
      return ExpectedSnafu {
        what: "a function declaration",
        span: stream.span(),
      }.fail();
    };

    Ok(Some(Self { decl }))
  }
}
