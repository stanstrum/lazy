use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Expression,
  QualifiedName,
};

use crate::tokenizer::{
  Grouping,
  GroupingType,
  Punctuation,
  Span,
  GetSpan,
  TokenEnum,
};

use crate::asterizer::error::ExpectedSnafu;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct StructInitializerChild {
  pub(crate) name: String,
  pub(crate) value: Box<Expression>,
  pub(crate) span: Span,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct StructInitializer {
  pub(crate) name: QualifiedName,
  pub(crate) children: Vec<StructInitializerChild>,
  pub(crate) span: Span,
}

impl GetSpan for StructInitializerChild {
  fn get_span(&self) -> &Span {
    &self.span
  }
}

impl GetSpan for StructInitializer {
  fn get_span(&self) -> &Span {
    &self.span
  }
}

impl MakeAst for StructInitializerChild {
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

    let Some(value) = stream.make()? else {
      return ExpectedSnafu {
        what: "an expression",
        span: stream.span(),
      }.fail();
    };

    Ok(Some(Self {
      name,
      value: Box::new(value),
      span: stream.span_mark(),
    }))
  }
}

impl MakeAst for StructInitializer {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
        let Some(name) = stream.make()? else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::CurlyBrace))) = stream.next_variant() else {
      // This could be a legit AtomExpression here -- just not an initializer,
      // so don't throw an error quite yet
      return Ok(None);
    };

    let mut children = vec![];
    loop {
      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace))) = stream.peek_variant() {
        stream.seek();

        break;
      };

      let Some(child) = stream.make()? else {
        return Ok(None);
      };

      children.push(child);
      stream.skip_whitespace_and_comments();

      match stream.next_variant() {
        Some(TokenEnum::Punctuation(Punctuation::Comma)) => {},
        Some(TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace))) => break,
        _ => return ExpectedSnafu {
          what: "a comma or a closing curly brace",
          span: stream.span(),
        }.fail()
      };
    };

    Ok(Some(Self {
      name,
      children,
      span: stream.span_mark(),
     }))
  }
}
