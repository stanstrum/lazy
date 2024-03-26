use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Expression
};

use crate::tokenizer::{
  TokenEnum,
  GroupingType,
  Grouping
};

use crate::asterizer::error::ExpectedSnafu;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct SubExpression {
  pub expr: Box<Expression>
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
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::Parenthesis))) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "a closing parenthesis",
      }.fail();
    };

    Ok(Some(Self { expr }))
  }
}
