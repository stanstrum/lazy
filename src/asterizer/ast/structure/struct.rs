use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  Type,
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

use crate::asterizer::error::*;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct StructMember {
  pub(crate) name: String,
  pub(crate) ty: Type,
  pub(crate) span: Span,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Struct {
  pub(crate) name: String,
  pub(crate) members: Vec<StructMember>,
  pub(crate) span: Span,
}

impl GetSpan for StructMember {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for Struct {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl MakeAst for StructMember {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
        let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      return Ok(None);
    };
    let name = name.to_owned();

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Punctuation(Punctuation::Colon)) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "a colon",
        span: stream.span(),
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(ty) = stream.make()? else {
      return ExpectedSnafu {
        what: "a type",
        span: stream.span(),
      }.fail();
    };

    Ok(Some(Self {
      name,
      ty,
      span: stream.span_mark(),
    }))
  }
}

impl MakeAst for Struct {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::Struct)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      let span = stream.span();

      return ExpectedSnafu {
        what: "an identifier",
        span,
      }.fail();
    };
    let name = name.to_owned();

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::CurlyBrace))) = stream.next_variant() else {
      let span = stream.span();

      return ExpectedSnafu {
        what: "an identifier",
        span,
      }.fail();
    };

    let mut members = vec![];
    loop {
      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace))) = stream.peek_variant() {
        stream.seek();

        break;
      };

      let Some(member) = stream.make()? else {
        return ExpectedSnafu {
          what: "a struct member",
          span: stream.span(),
        }.fail();
      };

      members.push(member);

      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Punctuation(Punctuation::Comma)) = stream.peek_variant() {
        stream.seek();

        continue;
      };

      let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace))) = stream.next_variant() else {
        return ExpectedSnafu {
          what: "a closing brace",
          span: stream.span(),
        }.fail();
      };

      break;
    };

    Ok(Some(Self {
      name,
      members,
      span: stream.span_mark(),
    }))
  }
}
