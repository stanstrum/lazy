use crate::asterizer::{
  TokenStream,
  AsterizerError,
  MakeAst
};

use super::Namespace;

#[derive(Debug)]
pub(crate) enum Structure {
  Namespace(Namespace),
}

impl MakeAst for Structure {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    todo!()
  }
}
