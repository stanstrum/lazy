use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  FunctionDeclaration,
  TypeAlias,
  Block,
};

use crate::tokenizer::{
  Grouping,
  GroupingType,
  Keyword,
  Punctuation,
  Span,
  GetSpan,
  TokenEnum,
};

use crate::asterizer::error::ExpectedSnafu;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Method {
  pub(crate) decl: FunctionDeclaration,
  pub(crate) body: Option<Block>,
  pub(crate) span: Span,
}

#[derive(Debug, TypeName)]
pub(crate) enum InterfaceChild {
  TypeAlias(TypeAlias),
  Method(Method),
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Interface {
  pub(crate) name: String,
  pub(crate) children: Vec<InterfaceChild>,
  pub(crate) span: Span,
}

impl GetSpan for Method {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for InterfaceChild {
  fn get_span(&self) -> Span {
    match self {
      InterfaceChild::TypeAlias(typealias) => typealias.get_span(),
      InterfaceChild::Method(method) => method.get_span(),
    }
  }
}

impl GetSpan for Interface {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl MakeAst for InterfaceChild {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    #[allow(clippy::manual_map)]
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

    Ok(Some(Self {
      decl,
      body,
      span: stream.span_mark(),
    }))
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
        span: stream.span()
      }.fail();
    };

    let name = name.to_owned();

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::CurlyBrace))) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "an opening curly brace",
        span: stream.span()
      }.fail();
    };

    let mut children = vec![];
    loop {
      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace))) = stream.peek_variant() {
        stream.seek();

        break;
      };

      let Some(child) = stream.make()? else {
        return ExpectedSnafu {
          what: "an interface child",
          span: stream.span()
        }.fail();
      };

      children.push(child);

      stream.skip_whitespace_and_comments();

      let Some(TokenEnum::Punctuation(Punctuation::Semicolon)) = stream.next_variant() else {
        return ExpectedSnafu {
          what: "a semicolon",
          span: stream.span()
        }.fail();
      };
    };

    Ok(Some(Self {
      name,
      children,
      span: stream.span_mark(),
    }))
  }
}
