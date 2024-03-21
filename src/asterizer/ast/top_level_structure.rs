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
      TopLevelStructure::Namespace(ns) => ns.name.to_owned(),
      TopLevelStructure::Function(func) => func.decl.name.to_owned(),
      TopLevelStructure::TypeAlias(alias) => alias.name.to_owned(),
    }
  }
}

impl MakeAst for TopLevelStructure {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    println!("TopLevelStructure::make");

    Ok({
      if let Some(ns) = stream.make::<Namespace>()? {
        Some(TopLevelStructure::Namespace(ns))
      } else if let Some(func) = stream.make::<Function>()? {
        Some(TopLevelStructure::Function(func))
      } else if let Some(alias) = stream.make::<TypeAlias>()? {
        Some(TopLevelStructure::TypeAlias(alias))
      } else {
        None
      }
    })
  }
}
