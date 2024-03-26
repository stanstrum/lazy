use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
};

use crate::tokenizer::Literal;

#[derive(Debug, TypeName)]
pub(crate) enum Atom {
  Literal(Literal),
}

impl MakeAst for Atom {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(literal) = stream.make()? {
        Some(Self::Literal(literal))
      } else {
        None
      }
    })
  }
}
