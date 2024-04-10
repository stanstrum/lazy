import_export!(namespace);
import_export!(type_alias);
import_export!(r#interface);

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Function,
};

#[derive(Debug, TypeName)]
pub(crate) enum Structure {
  Namespace(Namespace),
  Function(Function),
  TypeAlias(TypeAlias),
  Interface(Interface),
}

impl Structure {
  pub fn name(&self) -> String {
    match self {
      Self::Namespace(ns) => ns.name.to_owned(),
      Self::Function(func) => func.decl.name.to_owned(),
      Self::TypeAlias(alias) => alias.name.to_owned(),
      Self::Interface(r#interface) => r#interface.name.to_owned(),
    }
  }
}

impl MakeAst for Structure {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(ns) = stream.make()? {
        Some(Self::Namespace(ns))
      } else if let Some(func) = stream.make()? {
        Some(Self::Function(func))
      } else if let Some(alias) = stream.make()? {
        Some(Self::TypeAlias(alias))
      } else if let Some(alias) = stream.make()? {
        Some(Self::Interface(alias))
      } else {
        None
      }
    })
  }
}
