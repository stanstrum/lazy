use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError
};

use crate::tokenizer::{
  TokenEnum,
  Literal
};

impl MakeAst for Literal {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      match stream.next_variant() {
        Some(TokenEnum::Literal(literal)) => Some(literal.to_owned()),
        _ => None
      }
    })
  }
}
