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
  Punctuation,
};

use crate::asterizer::error::ExpectedSnafu;

#[derive(Debug, TypeName)]
pub(crate) struct Extern {
  pub(crate) decl: FunctionDeclaration,
  #[allow(unused)]
  pub(crate) c_variadic: bool,
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

    stream.push_mark();

    stream.skip_whitespace_and_comments();

    let c_variadic = {
      if let Some(TokenEnum::Punctuation(Punctuation::VariadicEllipsis)) = dbg!(stream.next_variant()) {
        stream.drop_mark();

        true
      } else {
        stream.pop_mark();

        false
      }
    };

    Ok(Some(Self { decl, c_variadic }))
  }
}
