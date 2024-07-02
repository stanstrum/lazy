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
pub(crate) enum BinaryOperatorKind {
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
#[derive(Debug, TypeName)]
pub(crate) struct BinaryOperator {
  pub(crate) kind: BinaryOperatorKind,
  pub(crate) span: Span,
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
    &self.span
  }
}

impl GetSpan for BinaryExpression {
  fn get_span(&self) -> &Span {
    &self.span
  }
}

impl MakeAst for BinaryOperator {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(variant) = stream.next_variant() else {
      return Ok(None);
    };

    let kind = match variant {
      TokenEnum::Operator(Operator::Add) => BinaryOperatorKind::Add,
      TokenEnum::Operator(Operator::AddAssign) => BinaryOperatorKind::AddAssign,
      TokenEnum::Operator(Operator::Subtract) => BinaryOperatorKind::Subtract,
      TokenEnum::Operator(Operator::SubtractAssign) => BinaryOperatorKind::SubtractAssign,
      TokenEnum::Operator(Operator::Exponent) => BinaryOperatorKind::Exponent,
      TokenEnum::Operator(Operator::ExponentAssign) => BinaryOperatorKind::ExponentAssign,
      TokenEnum::Operator(Operator::Multiply) => BinaryOperatorKind::Multiply,
      TokenEnum::Operator(Operator::MultiplyAssign) => BinaryOperatorKind::MultiplyAssign,
      TokenEnum::Operator(Operator::Divide) => BinaryOperatorKind::Divide,
      TokenEnum::Operator(Operator::DivideAssign) => BinaryOperatorKind::DivideAssign,
      TokenEnum::Operator(Operator::Modulo) => BinaryOperatorKind::Modulo,
      TokenEnum::Operator(Operator::ModuloAssign) => BinaryOperatorKind::ModuloAssign,
      TokenEnum::Operator(Operator::Equality) => BinaryOperatorKind::Comparison,
      TokenEnum::Operator(Operator::LessThan) => BinaryOperatorKind::LessThan,
      TokenEnum::Operator(Operator::LessThanEqual) => BinaryOperatorKind::LessThanEqual,
      TokenEnum::Operator(Operator::GreaterThan) => BinaryOperatorKind::GreaterThan,
      TokenEnum::Operator(Operator::GreaterThanEqual) => BinaryOperatorKind::GreaterThanEqual,
      TokenEnum::Operator(Operator::Equals) => BinaryOperatorKind::Equals,
      TokenEnum::Operator(Operator::Dot) => BinaryOperatorKind::Dot,
      TokenEnum::Operator(Operator::RightArrow) => BinaryOperatorKind::DerefDot,
      TokenEnum::Operator(Operator::Separator) => BinaryOperatorKind::Separator,
      TokenEnum::Operator(Operator::SingleAnd) => BinaryOperatorKind::BitwiseAnd,
      TokenEnum::Operator(Operator::SingleAndAssign) => BinaryOperatorKind::BitwiseAndAssign,
      // TokenEnum::Operator(Operator::SingleOr) => BinaryOperatorKind::BitwiseOr,
      // TokenEnum::Operator(Operator::SingleOrAssign) => BinaryOperatorKind::BitwiseOrAssign,
      // TokenEnum::Operator(Operator::SingleXor) => BinaryOperatorKind::BitwiseXor,
      // TokenEnum::Operator(Operator::SingleXorAssign) => BinaryOperatorKind::BitwiseXorAssign,
      TokenEnum::Operator(Operator::DoubleAnd) => BinaryOperatorKind::LogicalAnd,
      TokenEnum::Operator(Operator::DoubleAndAssign) => BinaryOperatorKind::LogicalAndAssign,
      // TokenEnum::Operator(Operator::DoubleOr) => BinaryOperatorKind::LogicalOr,
      // TokenEnum::Operator(Operator::DoubleOrAssign) => BinaryOperatorKind::LogicalOrAssign,
      // TokenEnum::Operator(Operator::DoubleXor) => BinaryOperatorKind::LogicalXor,
      // TokenEnum::Operator(Operator::DoubleXorAssign) => BinaryOperatorKind::LogicalXorAssign,
      _ => return Ok(None),
    };

    Ok(Some(Self {
      kind,
      span: stream.span_mark(),
    }))
  }
}
