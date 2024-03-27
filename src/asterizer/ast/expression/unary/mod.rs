import_export!(prefix);
import_export!(suffix);

use crate::asterizer::ast::Expression;

#[derive(Debug)]
pub(crate) enum UnaryOperator {
  Prefix(UnaryPrefixOperator),
  Suffix(UnarySuffixOperator)
}

#[derive(Debug)]
pub(crate) struct UnaryExpression {
  pub op: UnaryOperator,
  pub expr: Box<Expression>
}
