use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Expression,
};

use crate::tokenizer::{Operator, TokenEnum};

#[derive(Debug, TypeName)]
pub(crate) enum UnaryPrefixOperator {
  PreIncrement,
  PreDecrement,
}

#[derive(Debug)]
pub(crate) struct UnaryPrefixExpression {
  pub op: UnaryPrefixOperator,
  pub expr: Box<Expression>
}

impl MakeAst for UnaryPrefixOperator {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      match stream.next_variant() {
        Some(TokenEnum::Operator(Operator::Increment)) => Some(UnaryPrefixOperator::PreIncrement),
        Some(TokenEnum::Operator(Operator::Decrement)) => Some(UnaryPrefixOperator::PreDecrement),
        _ => None
      }
    })
  }
}
