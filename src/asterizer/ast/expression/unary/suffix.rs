use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Expression,
};

use crate::tokenizer::{
  Operator,
  TokenEnum,
};

#[derive(Debug, TypeName)]
pub(crate) enum UnarySuffixOperator {
  PostIncrement,
  PostDecrement
}

#[derive(Debug)]
pub(crate) struct UnarySuffixExpression {
  pub op: UnarySuffixOperator,
  pub expr: Box<Expression>
}

impl MakeAst for UnarySuffixOperator {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      match stream.next_variant() {
        Some(TokenEnum::Operator(Operator::Increment)) => Some(UnarySuffixOperator::PostIncrement),
        Some(TokenEnum::Operator(Operator::Decrement)) => Some(UnarySuffixOperator::PostDecrement),
        _ => None
      }
    })
  }
}

