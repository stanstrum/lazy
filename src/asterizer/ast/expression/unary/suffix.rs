use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Expression,
  Type,
};

use crate::tokenizer::{
  Grouping,
  GroupingType,
  Operator,
  Punctuation,
  Span,
  GetSpan,
  TokenEnum,
};

use crate::asterizer::error::ExpectedSnafu;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum UnarySuffixOperator {
  PostIncrement,
  PostDecrement,
  Call { args: Vec<Expression> },
  Subscript { expr: Box<Expression> },
  Cast { ty: Box<Type> },
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct UnarySuffixExpression {
  pub(crate) op: UnarySuffixOperator,
  pub(crate) expr: Box<Expression>,
  pub(crate) span: Span,
}

impl GetSpan for UnarySuffixOperator {
  fn get_span(&self) -> Span {
    todo!()
  }
}

impl GetSpan for UnarySuffixExpression {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl MakeAst for UnarySuffixOperator {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    match stream.next_variant() {
      Some(TokenEnum::Operator(Operator::Increment)) => Ok(Some(UnarySuffixOperator::PostIncrement)),
      Some(TokenEnum::Operator(Operator::Decrement)) => Ok(Some(UnarySuffixOperator::PostDecrement)),
      Some(TokenEnum::Grouping(Grouping::Open(GroupingType::Parenthesis))) => {
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
              span: stream.span()
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
              span: stream.span()
            }.fail();
          };

          break;
        };

        Ok(Some(Self::Call { args }))
      },
      Some(TokenEnum::Grouping(Grouping::Open(GroupingType::Bracket))) => {
        stream.skip_whitespace_and_comments();

        let Some(expr) = stream.make()? else {
          return ExpectedSnafu {
            what: "an expression",
            span: stream.span(),
          }.fail();
        };

        stream.skip_whitespace_and_comments();

        let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::Bracket))) = stream.next_variant() else {
          return ExpectedSnafu {
            what: "a closing bracket",
            span: stream.span(),
          }.fail();
        };

        Ok(Some(Self::Subscript {
          expr: Box::new(expr),
        }))
      },
      Some(TokenEnum::Punctuation(Punctuation::Colon)) => {
        stream.skip_whitespace_and_comments();

        let Some(ty) = stream.make()? else {
          return ExpectedSnafu {
            what: "a type",
            span: stream.span(),
          }.fail();
        };

        Ok(Some(Self::Cast {
          ty: Box::new(ty),
        }))
      },
      _ => Ok(None)
    }
  }
}
