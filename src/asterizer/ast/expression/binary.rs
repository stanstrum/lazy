use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Expression,
};

use crate::tokenizer::{
  TokenEnum,
  Operator,
};

#[derive(Debug, TypeName)]
pub(crate) enum BinaryOperator {
  Add,
  Subtract,
  Exponent,
  Multiply,
  Divide,
  Comparison,
  Equals,
  Dot,
  DerefDot,
  Separator,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct BinaryExpression {
  pub(crate) op: BinaryOperator,
  pub(crate) lhs: Box<Expression>,
  pub(crate) rhs: Box<Expression>,
}

impl MakeAst for BinaryOperator {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      match stream.next_variant() {
        Some(TokenEnum::Operator(Operator::Add)) => Some(Self::Add),
        Some(TokenEnum::Operator(Operator::Subtract)) => Some(Self::Subtract),
        Some(TokenEnum::Operator(Operator::Exponent)) => Some(Self::Exponent),
        Some(TokenEnum::Operator(Operator::Multiply)) => Some(Self::Multiply),
        Some(TokenEnum::Operator(Operator::Divide)) => Some(Self::Divide),
        Some(TokenEnum::Operator(Operator::Equality)) => Some(Self::Comparison),
        Some(TokenEnum::Operator(Operator::LessThan)) => Some(Self::Comparison),
        Some(TokenEnum::Operator(Operator::LessThanEqual)) => Some(Self::Comparison),
        Some(TokenEnum::Operator(Operator::GreaterThan)) => Some(Self::Comparison),
        Some(TokenEnum::Operator(Operator::GreaterThanEqual)) => Some(Self::Comparison),
        Some(TokenEnum::Operator(Operator::Equals)) => Some(Self::Equals),
        Some(TokenEnum::Operator(Operator::Dot)) => Some(Self::Dot),
        Some(TokenEnum::Operator(Operator::RightArrow)) => Some(Self::DerefDot),
        Some(TokenEnum::Operator(Operator::Separator)) => Some(Self::Separator),
        _ => None
      }
    })
  }
}
