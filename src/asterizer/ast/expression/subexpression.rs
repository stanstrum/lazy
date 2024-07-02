use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Expression,
};

use crate::tokenizer::{
  TokenEnum,
  GroupingType,
  Grouping,
  Span,
  GetSpan,
};

use crate::asterizer::error::ExpectedSnafu;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct SubExpression {
  pub(crate) expr: Box<Expression>,
  pub(crate) span: Span,
}

impl GetSpan for SubExpression {
  fn get_span(&self) -> &Span {
    &self.span
  }
}

impl MakeAst for SubExpression {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
        let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::Parenthesis))) = stream.next_variant() else {
      return Ok(None)
    };

    stream.skip_whitespace_and_comments();

    let Some(expr) = stream.make_boxed::<Expression>()? else {
      return ExpectedSnafu {
        what: "an expression",
        span: stream.span()
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::Parenthesis))) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "a closing parenthesis",
        span: stream.span()
      }.fail();
    };

    Ok(Some(Self {
      expr,
      span: stream.span_mark(),
    }))
  }
}
