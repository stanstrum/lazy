use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  Type,
};

use crate::tokenizer::{
  Punctuation,
  Span,
  GetSpan,
  TokenEnum,
};

use crate::asterizer::error::*;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct FunctionDeclarationArgument {
  pub(crate) name: String,
  pub(crate) ty: Type,
  pub(crate) span: Span,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct FunctionDeclarationArguments {
  pub(crate) args: Vec<FunctionDeclarationArgument>,
}

impl GetSpan for FunctionDeclarationArgument {
  fn get_span(&self) -> &Span {
    todo!()
  }
}

impl GetSpan for FunctionDeclarationArguments {
  fn get_span(&self) -> &Span {
    todo!()
  }
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
        span: stream.span()
      }.fail();
    };

    Ok(Some(Self {
      name,
      ty,
      span: stream.span_mark(),
    }))
  }
}

impl MakeAst for FunctionDeclarationArguments {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let mut args = vec![];

    loop {
      stream.skip_whitespace_and_comments();

      let Some(arg) = stream.make()? else {
        return ExpectedSnafu {
          what: "a function declaration argument",
          span: stream.span(),
        }.fail();
      };

      args.push(arg);

      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Punctuation(Punctuation::Comma)) = stream.peek_variant() {
        stream.seek();

        stream.push_mark();
        stream.skip_whitespace_and_comments();

        if let Some(TokenEnum::Punctuation(Punctuation::VariadicEllipsis)) = stream.peek_variant() {
          stream.pop_mark();

          break;
        } else {
          stream.drop_mark();
        };
      } else {
        break;
      };
    };

    Ok(Some(Self {
      args
    }))
  }
}
