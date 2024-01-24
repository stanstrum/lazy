use crate::asterizer::{
  TokenStream,
  AsterizerError,
  MakeAst
};

use super::{
  Namespace,
  Function
};

#[derive(Debug)]
pub(crate) enum TopLevelStructure {
  Namespace(Namespace),
  Function(Function)
}

impl TopLevelStructure {
  pub fn name(&self) -> String {
    match self {
      TopLevelStructure::Namespace(ns) => ns.name.to_owned(),
      TopLevelStructure::Function(func) => func.name.to_owned(),
    }
  }
}

impl MakeAst for TopLevelStructure {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(ns) = Namespace::make(stream)? {
        Some(TopLevelStructure::Namespace(ns))
      } else if let Some(func) = Function::make(stream)? {
        Some(TopLevelStructure::Function(func))
      } else {
        None
      }
    })
  }
}
