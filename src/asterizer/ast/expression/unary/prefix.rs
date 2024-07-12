use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Expression,
};

use crate::tokenizer::{
  Keyword,
  Operator,
  Span,
  GetSpan,
  TokenEnum,
};

#[derive(Debug)]
pub(crate) enum UnaryPrefixOperatorKind {
  PreIncrement,
  PreDecrement,
  ImpliedSeparator,
  Reference,
  MutReference,
}

#[derive(Debug, TypeName)]
pub(crate) struct UnaryPrefixOperator {
  pub(crate) kind: UnaryPrefixOperatorKind,
  pub(crate) span: Span,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct UnaryPrefixExpression {
  pub(crate) op: UnaryPrefixOperator,
  pub(crate) expr: Box<Expression>,
  pub(crate) span: Span,
}

impl GetSpan for UnaryPrefixOperator {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for UnaryPrefixExpression {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl MakeAst for UnaryPrefixOperator {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let kind = {
      match stream.next_variant() {
        Some(TokenEnum::Operator(Operator::Increment)) => UnaryPrefixOperatorKind::PreIncrement,
        Some(TokenEnum::Operator(Operator::Decrement)) => UnaryPrefixOperatorKind::PreDecrement,
        Some(TokenEnum::Operator(Operator::Separator)) => UnaryPrefixOperatorKind::ImpliedSeparator,
        Some(TokenEnum::Operator(Operator::SingleAnd)) => {
          stream.push_mark();
          stream.skip_whitespace_and_comments();

          if let Some(TokenEnum::Keyword(Keyword::Mut)) = stream.next_variant() {
            stream.drop_mark();

            UnaryPrefixOperatorKind::MutReference
          } else {
            stream.pop_mark();

            UnaryPrefixOperatorKind::Reference
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
