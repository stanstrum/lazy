use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Expression,
};

use crate::asterizer::error::ExpectedSnafu;
use crate::tokenizer::{
  Grouping,
  GroupingType,
  Operator,
  Punctuation,
  TokenEnum,
};

#[derive(Debug, TypeName)]
pub(crate) enum UnarySuffixOperator {
  PostIncrement,
  PostDecrement,
  Call { args: Vec<Expression> }
}

#[derive(Debug)]
pub(crate) struct UnarySuffixExpression {
  pub op: UnarySuffixOperator,
  pub expr: Box<Expression>
}

impl MakeAst for UnarySuffixOperator {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let simple_operator = {
      match stream.peek_variant() {
        Some(TokenEnum::Operator(Operator::Increment)) => Some(UnarySuffixOperator::PostIncrement),
        Some(TokenEnum::Operator(Operator::Decrement)) => Some(UnarySuffixOperator::PostDecrement),
        _ => None
      }
    };

    if simple_operator.is_some() {
      stream.seek();

      return Ok(simple_operator);
    };

    if let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::Parenthesis))) = stream.peek_variant() {
      stream.seek();

      let mut args = vec![];

      loop {
        stream.skip_whitespace_and_comments();

        if args.is_empty() {
          if let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::Parenthesis))) = stream.peek_variant() {
            stream.seek();

            break;
          };
        };

        let Some(arg) = stream.make::<Expression>()? else {
          return ExpectedSnafu {
            what: "an expression",
          }.fail();
        };

        args.push(arg);

        stream.skip_whitespace_and_comments();

        if let Some(TokenEnum::Punctuation(Punctuation::Comma)) = stream.peek_variant() {
          stream.seek();

          continue;
        };

        let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::Parenthesis))) = stream.next_variant() else {
          return ExpectedSnafu {
            what: "a closing parenthesis",
          }.fail();
        };

        break;
      };

      return Ok(Some(Self::Call { args }));
    };

    Ok(None)
  }
}

