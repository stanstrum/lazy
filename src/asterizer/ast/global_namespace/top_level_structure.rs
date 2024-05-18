use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Structure,
};

#[derive(Debug, TypeName)]
pub(crate) enum TopLevelStructure {
  // import, export, etc.
  Structure(Structure),
}

impl TopLevelStructure {
  pub fn name(&self) -> String {
    match self {
      Self::Structure(structure) => structure.name(),
    }
  }
}

impl MakeAst for TopLevelStructure {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    #[allow(clippy::manual_map)]
    Ok({
      if let Some(structure) = stream.make()? {
        Some(Self::Structure(structure))
      } else {
        None
      }
    })
  }
}
