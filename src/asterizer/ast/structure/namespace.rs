use typename::TypeName;

use crate::tokenizer::{
  Grouping,
  GroupingType,
  Keyword,
  Punctuation,
  Span,
  GetSpan,
  TokenEnum,
};

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Structure,
};

use crate::asterizer::error::ExpectedSnafu;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Namespace {
  pub(crate) name: String,
  pub(crate) children: Vec<Structure>,
  pub(crate) span: Span,
}

impl GetSpan for Namespace {
  fn get_span(&self) -> &Span {
    &self.span
  }
}

impl MakeAst for Namespace {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
        let Some(TokenEnum::Keyword(Keyword::Namespace)) = stream.next_variant() else {
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

      let Some(structure) = stream.make::<Structure>()? else {
        return ExpectedSnafu {
          what: "a structure",
          span: stream.span()
        }.fail();
      };

      children.push(structure);

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
