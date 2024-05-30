mod pemdas;

use crate::asterizer::ast::{
  TokenStream,
  AsterizerError,
  Expression,
  BinaryOperator,
  UnaryOperator,
  NonOperatorExpression,
};

#[derive(Debug)]
pub(super) enum ExpressionPart {
  Unary(UnaryOperator),
  Binary(BinaryOperator),
  Operand(Expression),
}

pub(super) struct ExpressionResolver<'a> {
  pub(crate) stream: &'a mut TokenStream,
  pub(crate) parts: Vec<ExpressionPart>
}

impl<'a> ExpressionResolver<'a> {
  pub fn new(stream: &'a mut TokenStream) -> Self {
    Self {
      stream,
      parts: vec![],
    }
  }

  pub fn make_binary_operator(&mut self) -> Result<Option<()>, AsterizerError> {
    let Some(op) = self.stream.make()? else {
      return Ok(None);
    };

    self.parts.push(ExpressionPart::Binary(op));

    Ok(Some(()))
  }

  pub fn make_binary_part(&mut self) -> Result<Option<()>, AsterizerError> {
    loop {
      let Some(op) = self.stream.make()? else {
        break;
      };

      self.parts.push(ExpressionPart::Unary(UnaryOperator::Prefix(op)));

      self.stream.skip_whitespace_and_comments();
    };

    let Some(expr) = self.stream.make::<NonOperatorExpression>()? else {
      return Ok(None);
    };

    self.parts.push(ExpressionPart::Operand(expr.into()));

    loop {
      self.stream.push_mark();
      self.stream.skip_whitespace_and_comments();

      let Some(op) = self.stream.make()? else {
        self.stream.pop_mark();
        break;
      };

      self.stream.drop_mark();

      self.parts.push(ExpressionPart::Unary(UnaryOperator::Suffix(op)));
    };

    Ok(Some(()))
  }
}
