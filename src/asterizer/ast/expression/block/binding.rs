use typename::TypeName;

use crate::tokenizer::{
  TokenEnum,
  Operator,
  Punctuation,
};

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  Expression,
  Type
};

use crate::asterizer::error::*;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Binding {
  // r#mut: bool,
  name: String,
  ty: Option<Type>,
  expr: Option<Expression>
}

impl MakeAst for Binding {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      return Ok(None);
    };

    let name = name.to_owned();

    stream.skip_whitespace_and_comments();

    let ty = {
      if let Some(TokenEnum::Punctuation(Punctuation::Colon)) = stream.peek_variant() {
        stream.seek();
        stream.skip_whitespace_and_comments();

        let Some(ty) = stream.make()? else {
          return ExpectedSnafu {
            what: "a type",
          }.fail();
        };

        Some(ty)
      } else {
        None
      }
    };

    stream.skip_whitespace_and_comments();

    let expr = {
      if let Some(TokenEnum::Operator(Operator::BindAssign)) = stream.peek_variant() {
        stream.seek();
        stream.skip_whitespace_and_comments();

        let Some(expr) = stream.make()? else {
          return ExpectedSnafu {
            what: "an expression",
          }.fail();
        };

        Some(expr)
      } else {
        None
      }
    };

    if let (None, None) = (&ty, &expr) {
      return ExpectedSnafu {
        what: "a type or an expression",
      }.fail();
    };

    Ok(Some(Self { name, ty, expr }))
  }
}
