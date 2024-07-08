use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Type,
  Block,
  TemplateScope,
  MethodArguments,
};

use crate::tokenizer::{
  Grouping,
  GroupingType,
  Keyword,
  Operator,
  Punctuation,
  Span,
  GetSpan,
  TokenEnum,
};

use crate::asterizer::error::ExpectedSnafu;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum ImplKind {
  Impl { what: Type },
  ImplFor { what: Type, r#trait: Type },
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct ImplMethod {
  pub(crate) template: Option<TemplateScope>,
  pub(crate) name: String,
  pub(crate) return_ty: Option<Type>,
  pub(crate) args: MethodArguments,
  pub(crate) body: Block,
  pub(crate) span: Span,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Impl {
  pub(crate) kind: ImplKind,
  pub(crate) children: Vec<ImplMethod>,
  pub(crate) span: Span,
}

impl GetSpan for ImplKind {
  fn get_span(&self) -> Span {
    todo!()
  }
}

impl GetSpan for ImplMethod {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for Impl {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl MakeAst for ImplMethod {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
        let template = stream.make()?;

    if template.is_some() {
      stream.skip_whitespace_and_comments();
    };

    let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      return Ok(None);
    };
    let name = name.to_owned();

    let return_ty = {
      stream.push_mark();
      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Operator(Operator::RightArrow)) = stream.next_variant() {
        stream.skip_whitespace_and_comments();
        stream.drop_mark();

        let Some(ty) = stream.make()? else {
          return ExpectedSnafu {
            what: "a type",
            span: stream.span(),
          }.fail();
        };

        Some(ty)
      } else {
        stream.pop_mark();

        None
      }
    };

    stream.skip_whitespace_and_comments();

    let args = stream.make()?.expect("method args failed");

    stream.skip_whitespace_and_comments();

    let Some(body) = stream.make()? else {
      return ExpectedSnafu {
        what: "a block expression",
        span: stream.span(),
      }.fail();
    };

    Ok(Some(Self {
      template,
      name,
      return_ty,
      args,
      body,
      span: stream.span_mark(),
    }))
  }
}

impl MakeAst for Impl {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::Impl)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(what) = stream.make()? else {
      return ExpectedSnafu {
        what: "a type",
        span: stream.span(),
      }.fail();
    };

    let kind = {
      stream.push_mark();
      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Punctuation(Punctuation::Colon)) = stream.next_variant() {
        stream.drop_mark();
        stream.skip_whitespace_and_comments();

        let Some(r#trait) = stream.make()? else {
          return ExpectedSnafu {
            what: "a type",
            span: stream.span(),
          }.fail();
        };

        ImplKind::ImplFor { what, r#trait }
      } else {
        stream.pop_mark();

        ImplKind::Impl { what }
      }
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::CurlyBrace))) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "an opening curly brace",
        span: stream.span(),
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
          what: "an impl method",
          span: stream.span(),
        }.fail();
      };

      children.push(child);
      stream.skip_whitespace_and_comments();

      let Some(TokenEnum::Punctuation(Punctuation::Semicolon)) = stream.next_variant() else {
        return ExpectedSnafu {
          what: "a semicolon",
          span: stream.span(),
        }.fail();
      };
    };

    Ok(Some(Self {
      kind,
      children,
      span: stream.span_mark(),
    }))
  }
}
