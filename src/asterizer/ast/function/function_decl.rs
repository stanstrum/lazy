use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  FunctionDeclarationArguments,
  Type
};

use crate::tokenizer::{
  Token,
  TokenEnum,
  Operator,
  Punctuation
};

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct FunctionDeclaration {
  pub name: String,
  pub return_type: Option<Type>,
  pub args: Option<FunctionDeclarationArguments>,
}

impl MakeAst for FunctionDeclaration {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    println!("FunctionDecl::make");

    let Some(Token {
      token: TokenEnum::Identifier(ident),
      span: _span
    }) = stream.next() else {
      return Ok(None);
    };

    let name = ident.to_owned();

    stream.skip_whitespace_and_comments();

    stream.push_mark();

    let return_type = {
      if let Some(TokenEnum::Operator(Operator::RightArrow)) = stream.next_variant() {
        stream.skip_whitespace_and_comments();

        if let Some(ty) = stream.make()? {
          stream.drop_mark();

          Some(ty)
        } else {
          stream.pop_mark();

          None
        }
      } else {
        stream.pop_mark();

        None
      }
    };

    stream.skip_whitespace_and_comments();

    stream.push_mark();

    let args = {
      if let Some(TokenEnum::Punctuation(Punctuation::Colon)) = stream.next_variant() {
        match stream.make()? {
          Some(args) => {
            stream.drop_mark();

            Some(args)
          },
          None => {
            stream.pop_mark();

            None
          }
        }
      } else {
        stream.pop_mark();

        None
      }
    };

    Ok(Some(Self {
      name, return_type, args
    }))
  }
}
