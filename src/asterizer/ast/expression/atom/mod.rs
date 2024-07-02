import_export!(literal);
import_export!(struct_initializer);

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
};

use crate::tokenizer::{
  TokenEnum,
  Literal,
  Span,
  GetSpan,
};

impl GetSpan for Atom {
  fn get_span(&self) -> &Span {
    todo!()
  }
}

#[derive(Debug, TypeName)]
pub(crate) enum Atom {
  Literal(Literal),
  StructInitializer(StructInitializer),
  Variable(String),
}

impl MakeAst for Atom {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(literal) = stream.make()? {
        Some(Self::Literal(literal))
      } else if let Some(initializer) = stream.make()? {
        Some(Self::StructInitializer(initializer))
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
