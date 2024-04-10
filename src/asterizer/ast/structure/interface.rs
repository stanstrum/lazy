use typename::TypeName;

use crate::asterizer::ast::{
  AsterizerError,
  FunctionDeclaration,
  MakeAst,
  TokenStream,
  Block,
  TypeAlias,
};

use crate::tokenizer::{
  TokenEnum,
  Grouping,
  GroupingType,
  Keyword,
  Punctuation,
};

use crate::asterizer::error::ExpectedSnafu;

#[derive(Debug, TypeName)]
pub(crate) struct Method {
  decl: FunctionDeclaration,
  body: Option<Block>,
}

#[derive(Debug, TypeName)]
pub(crate) enum InterfaceChild {
  TypeAlias(TypeAlias),
  Method(Method),
}

#[derive(Debug, TypeName)]
pub(crate) struct Interface {
  pub name: String,
  pub children: Vec<InterfaceChild>
}

impl MakeAst for InterfaceChild {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(alias) = stream.make()? {
        Some(Self::TypeAlias(alias))
      } else if let Some(method) = stream.make()? {
        Some(Self::Method(method))
      } else {
        None
      }
    })
  }
}

impl MakeAst for Method {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(decl) = stream.make()? else {
      return Ok(None);
    };

    stream.push_mark();

    stream.skip_whitespace_and_comments();

    let body = stream.make()?;

    if body.is_some() {
      stream.drop_mark();
    } else {
      stream.pop_mark();
    };

    Ok(Some(Self { decl, body }))
  }
}

impl MakeAst for Interface {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::Interface)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "an identifier",
      }.fail();
    };

    let name = name.to_owned();

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::CurlyBrace))) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "an opening curly brace",
      }.fail();
    };

    let mut children = vec![];

    println!("A");
    loop {
      println!("B");

      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace))) = stream.peek_variant() {
        stream.seek();

        break;
      };

      let Some(child) = stream.make()? else {
        return ExpectedSnafu {
          what: "an interface child",
        }.fail();
      };

      children.push(child);

      stream.skip_whitespace_and_comments();

      let Some(TokenEnum::Punctuation(Punctuation::Semicolon)) = stream.next_variant() else {
        return ExpectedSnafu {
          what: "a semicolon",
        }.fail();
      };
    };

    Ok(Some(Self { name, children }))
  }
}
