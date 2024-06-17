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
  Keyword,
};

#[derive(Debug, TypeName)]
pub(crate) enum UnaryPrefixOperator {
  PreIncrement,
  PreDecrement,
  ImpliedSeparator,
  Reference,
  MutReference,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct UnaryPrefixExpression {
  pub(crate) op: UnaryPrefixOperator,
  pub(crate) expr: Box<Expression>
}

impl MakeAst for UnaryPrefixOperator {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      match stream.next_variant() {
        Some(TokenEnum::Operator(Operator::Increment)) => Some(UnaryPrefixOperator::PreIncrement),
        Some(TokenEnum::Operator(Operator::Decrement)) => Some(UnaryPrefixOperator::PreDecrement),
        Some(TokenEnum::Operator(Operator::Separator)) => Some(UnaryPrefixOperator::ImpliedSeparator),
        Some(TokenEnum::Operator(Operator::SingleAnd)) => {
          stream.push_mark();
          stream.skip_whitespace_and_comments();

          if let Some(TokenEnum::Keyword(Keyword::Mut)) = stream.next_variant() {
            stream.drop_mark();

            Some(UnaryPrefixOperator::MutReference)
          } else {
            stream.pop_mark();

            Some(UnaryPrefixOperator::Reference)
          }
        },
        _ => None
      }
    })
  }
}
