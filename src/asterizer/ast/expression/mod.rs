import_export!(block);
import_export!(subexpression);
import_export!(atom);
import_export!(unary);
import_export!(binary);

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

fn make_simple(stream: &mut TokenStream) -> Result<Option<Expression>, AsterizerError> {
  Ok({
    if let Some(block) = stream.make()? {
      Some(Expression::Block(block))
    } else if let Some(subexpr) = stream.make()? {
      Some(Expression::SubExpression(subexpr))
    } else if let Some(atom) = stream.make()? {
      Some(Expression::Atom(atom))
    } else {
      None
    }
  })
}

fn make_unary(stream: &mut TokenStream) -> Result<Option<Expression>, AsterizerError> {


  todo!()
}

impl MakeAst for Expression {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    todo!()
  }
}
