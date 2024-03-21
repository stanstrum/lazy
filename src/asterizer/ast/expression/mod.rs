mod block;
pub(crate) use block::*;

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError
};

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum Expression {
  BlockExpression(BlockExpression)
}

impl MakeAst for Expression {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(block) = stream.make::<BlockExpression>()? {
        Some(Expression::BlockExpression(block))
      } else {
        None
      }
    })
  }
}
