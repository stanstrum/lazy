mod block;
mod literal;
mod subexpression;
mod atom;

pub(crate) use block::*;
pub(crate) use subexpression::*;
pub(crate) use atom::*;

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError
};

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum Expression {
  Atom(Atom),
  Block(Block),
  SubExpression(SubExpression),
}

impl MakeAst for Expression {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(block) = stream.make()? {
        Some(Self::Block(block))
      } else if let Some(subexpr) = stream.make()? {
        Some(Self::SubExpression(subexpr))
      } else if let Some(atom) = stream.make()? {
        Some(Self::Atom(atom))
      } else {
        None
      }
    })
  }
}
