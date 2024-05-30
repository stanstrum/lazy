import_export!(literal);

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
};

use crate::tokenizer::{
  TokenEnum,
  Literal,
};

#[derive(Debug, TypeName)]
pub(crate) enum Atom {
  Literal(Literal),
  Variable(String),
}

impl MakeAst for Atom {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(literal) = stream.make()? {
        Some(Self::Literal(literal))
      } else if let Some(TokenEnum::Identifier(name)) = stream.peek_variant() {
        let name = name.to_owned();

        stream.seek();

        Some(Self::Variable(name))
      } else {
        None
      }
    })
  }
}
