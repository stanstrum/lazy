use crate::asterizer::{
  TokenStream,
  AsterizerError,
  MakeAst
};

#[derive(Debug)]
pub(crate) struct Function {
  pub name: String,
}

impl MakeAst for Function {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    todo!()
  }
}
