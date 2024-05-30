use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
};

use crate::tokenizer::{
  TokenEnum,
  Literal,
};

impl MakeAst for Literal {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Literal(literal)) = stream.next_variant() else {
      return Ok(None);
    };

    Ok(Some(literal.to_owned()))
  }
}
