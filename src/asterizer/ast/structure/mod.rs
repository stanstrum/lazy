import_export!(namespace);
import_export!(type_alias);

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError
};

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum Structure {
  Namespace(Namespace),
}

impl MakeAst for Structure {
  fn make(_stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    todo!()
  }
}
