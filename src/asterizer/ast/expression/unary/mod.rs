import_export!(prefix);
import_export!(suffix);

use crate::asterizer::ast::Expression;

use crate::tokenizer::{
  Span,
  GetSpan,
};

#[derive(Debug)]
pub(crate) enum UnaryOperator {
  Prefix(UnaryPrefixOperator),
  Suffix(UnarySuffixOperator),
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct UnaryExpression {
  pub(crate) op: UnaryOperator,
  pub(crate) expr: Box<Expression>,
  pub(crate) span: Span,
}

impl GetSpan for UnaryOperator {
  fn get_span(&self) -> &Span {
    todo!()
  }
}

impl GetSpan for UnaryExpression {
  fn get_span(&self) -> &Span {
    todo!()
  }
}

