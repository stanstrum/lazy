use typename::TypeName;

#[macro_use]
mod macros;

import_export!(global_namespace);
import_export!(structure);
import_export!(function);
import_export!(expression);
import_export!(ty);

use crate::asterizer::{
  TokenStream,
  AsterizerError,
};

pub(crate) trait MakeAst where Self: Sized + TypeName {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError>;
}
