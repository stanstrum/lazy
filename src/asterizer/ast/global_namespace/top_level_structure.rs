use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Namespace,
  Function,
  TypeAlias
};

#[derive(Debug, TypeName)]
pub(crate) enum TopLevelStructure {
  Namespace(Namespace),
  Function(Function),
  TypeAlias(TypeAlias)
}

impl TopLevelStructure {
  pub fn name(&self) -> String {
    match self {
      Self::Namespace(ns) => ns.name.to_owned(),
      Self::Function(func) => func.decl.name.to_owned(),
      Self::TypeAlias(alias) => alias.name.to_owned(),
    }
  }
}

impl MakeAst for TopLevelStructure {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    println!("TopLevelStructure::make");

    Ok({
      if let Some(ns) = stream.make()? {
        Some(Self::Namespace(ns))
      } else if let Some(func) = stream.make()? {
        Some(Self::Function(func))
      } else if let Some(alias) = stream.make()? {
        Some(Self::TypeAlias(alias))
      } else {
        None
      }
    })
  }
}
