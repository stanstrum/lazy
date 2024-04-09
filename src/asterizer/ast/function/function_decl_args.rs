use typename::TypeName;

use crate::tokenizer::{
  Punctuation,
  TokenEnum
};

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  Type
};

use crate::asterizer::error::*;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct FunctionDeclarationArgument {
  pub name: String,
  pub ty: Type
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct FunctionDeclarationArguments {
  pub args: Vec<FunctionDeclarationArgument>
}

impl MakeAst for FunctionDeclarationArgument {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      return Ok(None);
    };

    let name = name.to_owned();

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Punctuation(Punctuation::Colon)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(ty) = stream.make()? else {
      return ExpectedSnafu {
        what: "a type",
      }.fail();
    };

    Ok(Some(Self {
      name, ty,
    }))
  }
}

impl MakeAst for FunctionDeclarationArguments {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let mut args = vec![];

    loop {
      stream.skip_whitespace_and_comments();

      let Some(arg) = stream.make()? else {
        break;
      };

      args.push(arg);

      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Punctuation(Punctuation::Comma)) = stream.peek_variant() {
        stream.seek();
      } else {
        break;
      };
    };

    Ok(Some(Self {
      args
    }))
  }
}
