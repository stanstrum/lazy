use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Expression,
};

use crate::tokenizer::{
  Operator,
  Span,
  GetSpan,
  TokenEnum,
};

#[derive(Debug, TypeName)]
pub(crate) enum BinaryOperator {
  Add,
  AddAssign,
  Subtract,
  SubtractAssign,
  Exponent,
  ExponentAssign,
  Multiply,
  MultiplyAssign,
  Divide,
  DivideAssign,
  Modulo,
  ModuloAssign,
  Comparison,
  LessThan,
  LessThanEqual,
  GreaterThan,
  GreaterThanEqual,
  Equals,
  Dot,
  DerefDot,
  Separator,
  BitwiseAnd,
  BitwiseAndAssign,
  // BitwiseOr,
  // BitwiseOrAssign,
  // BitwiseXor,
  // BitwiseXorAssign,
  LogicalAnd,
  LogicalAndAssign,
  // LogicalOr,
  // LogicalOrAssign,
  // LogicalXor,
  // LogicalXorAssign,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct BinaryExpression {
  pub(crate) op: BinaryOperator,
  pub(crate) lhs: Box<Expression>,
  pub(crate) rhs: Box<Expression>,
  pub(crate) span: Span,
}

impl GetSpan for BinaryOperator {
  fn get_span(&self) -> &Span {
    todo!()
  }
}

impl GetSpan for BinaryExpression {
  fn get_span(&self) -> &Span {
    todo!()
  }
}

impl MakeAst for BinaryOperator {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      match stream.next_variant() {
        Some(TokenEnum::Operator(Operator::Add)) => Some(Self::Add),
        Some(TokenEnum::Operator(Operator::AddAssign)) => Some(Self::AddAssign),
        Some(TokenEnum::Operator(Operator::Subtract)) => Some(Self::Subtract),
        Some(TokenEnum::Operator(Operator::SubtractAssign)) => Some(Self::SubtractAssign),
        Some(TokenEnum::Operator(Operator::Exponent)) => Some(Self::Exponent),
        Some(TokenEnum::Operator(Operator::ExponentAssign)) => Some(Self::ExponentAssign),
        Some(TokenEnum::Operator(Operator::Multiply)) => Some(Self::Multiply),
        Some(TokenEnum::Operator(Operator::MultiplyAssign)) => Some(Self::MultiplyAssign),
        Some(TokenEnum::Operator(Operator::Divide)) => Some(Self::Divide),
        Some(TokenEnum::Operator(Operator::DivideAssign)) => Some(Self::DivideAssign),
        Some(TokenEnum::Operator(Operator::Modulo)) => Some(Self::Modulo),
        Some(TokenEnum::Operator(Operator::ModuloAssign)) => Some(Self::ModuloAssign),
        Some(TokenEnum::Operator(Operator::Equality)) => Some(Self::Comparison),
        Some(TokenEnum::Operator(Operator::LessThan)) => Some(Self::LessThan),
        Some(TokenEnum::Operator(Operator::LessThanEqual)) => Some(Self::LessThanEqual),
        Some(TokenEnum::Operator(Operator::GreaterThan)) => Some(Self::GreaterThan),
        Some(TokenEnum::Operator(Operator::GreaterThanEqual)) => Some(Self::GreaterThanEqual),
        Some(TokenEnum::Operator(Operator::Equals)) => Some(Self::Equals),
        Some(TokenEnum::Operator(Operator::Dot)) => Some(Self::Dot),
        Some(TokenEnum::Operator(Operator::RightArrow)) => Some(Self::DerefDot),
        Some(TokenEnum::Operator(Operator::Separator)) => Some(Self::Separator),
        Some(TokenEnum::Operator(Operator::SingleAnd)) => Some(Self::BitwiseAnd),
        Some(TokenEnum::Operator(Operator::SingleAndAssign)) => Some(Self::BitwiseAndAssign),
        // Some(TokenEnum::Operator(Operator::SingleOr)) => Some(Self::BitwiseOr),
        // Some(TokenEnum::Operator(Operator::SingleOrAssign)) => Some(Self::BitwiseOrAssign),
        // Some(TokenEnum::Operator(Operator::SingleXor)) => Some(Self::BitwiseXor),
        // Some(TokenEnum::Operator(Operator::SingleXorAssign)) => Some(Self::BitwiseXorAssign),
        Some(TokenEnum::Operator(Operator::DoubleAnd)) => Some(Self::LogicalAnd),
        Some(TokenEnum::Operator(Operator::DoubleAndAssign)) => Some(Self::LogicalAndAssign),
        // Some(TokenEnum::Operator(Operator::DoubleOr)) => Some(Self::LogicalOr),
        // Some(TokenEnum::Operator(Operator::DoubleOrAssign)) => Some(Self::LogicalOrAssign),
        // Some(TokenEnum::Operator(Operator::DoubleXor)) => Some(Self::LogicalXor),
        // Some(TokenEnum::Operator(Operator::DoubleXorAssign)) => Some(Self::LogicalXorAssign),
        _ => None
      }
    })
  }
}
