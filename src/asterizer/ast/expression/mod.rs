mod block;
mod literal;

pub(crate) use block::*;
pub(crate) use literal::*;

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
  Literal(Literal)
}

impl MakeAst for Expression {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(block) = stream.make::<Block>()? {
        Some(Expression::Block(block))
      } else if let Some(literal) = stream.make::<Literal>()? {
        Some(Expression::Literal(literal))
      } else {
        None
      }
    })
  }
}
