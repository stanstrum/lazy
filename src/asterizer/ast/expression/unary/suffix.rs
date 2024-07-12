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
pub(crate) enum UnarySuffixOperatorKind {
  PostIncrement,
  PostDecrement,
  Call { args: Vec<Expression> },
  Subscript { expr: Box<Expression> },
  Cast { ty: Box<Type> },
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct UnarySuffixOperator {
  pub(crate) kind: UnarySuffixOperatorKind,
  pub(crate) span: Span,
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
    self.span
  }
}

impl GetSpan for UnarySuffixExpression {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl MakeAst for UnarySuffixOperator {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let kind = {
      match stream.next_variant() {
        Some(TokenEnum::Operator(Operator::Increment)) => UnarySuffixOperatorKind::PostIncrement,
        Some(TokenEnum::Operator(Operator::Decrement)) => UnarySuffixOperatorKind::PostDecrement,
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

          UnarySuffixOperatorKind::Call { args }
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

          UnarySuffixOperatorKind::Subscript {
            expr: Box::new(expr),
          }
        },
        Some(TokenEnum::Punctuation(Punctuation::Colon)) => {
          stream.skip_whitespace_and_comments();

          let Some(ty) = stream.make()? else {
            return ExpectedSnafu {
              what: "a type",
              span: stream.span(),
            }.fail();
          };

          UnarySuffixOperatorKind::Cast {
            ty: Box::new(ty),
          }
        },
        _ => return Ok(None)
      }
    };

    Ok(Some(Self {
      kind,
      span: stream.span_mark(),
    }))
  }
}
