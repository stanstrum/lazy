mod block;
mod literal;
mod subexpression;

pub(crate) use block::*;
pub(crate) use subexpression::*;

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError
};

use crate::tokenizer::Literal;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum Expression {
  Block(Block),
  Literal(Literal),
  SubExpression(SubExpression),
}

impl MakeAst for Expression {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(block) = stream.make()? {
        Some(Self::Block(block))
      } else if let Some(subexpr) = stream.make()? {
        Some(Self::SubExpression(subexpr))
      } else if let Some(literal) = stream.make()? {
        Some(Self::Literal(literal))
      } else {
        None
      }
    })
  }
}
