import_export!(prefix);
import_export!(suffix);

use crate::asterizer::ast::Expression;

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
}
