use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  Type
};

use crate::tokenizer::{
  TokenEnum,
  Keyword,
  Operator
};

use crate::asterizer::error::*;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct TypeAlias {
  pub name: String,
  pub ty: Type
}

impl MakeAst for TypeAlias {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::Type)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "an identifier",
        span: stream.span()
      }.fail();
    };

    let name = name.to_owned();

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Operator(Operator::BindAssign)) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "the binding assignment operator",
        span: stream.span()
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(ty) = stream.make()? else {
      return ExpectedSnafu {
        what: "a type",
        span: stream.span()
      }.fail();
    };

    Ok(Some(Self {
      name, ty
    }))
  }
}
