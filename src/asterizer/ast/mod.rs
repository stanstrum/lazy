pub(self) mod namespace;
pub(self) mod global_namespace;
pub(self) mod top_level_structure;
pub(self) mod structure;

pub(crate) use namespace::*;
pub(crate) use global_namespace::*;
pub(crate) use top_level_structure::*;
pub(crate) use structure::*;

use super::{
  TokenStream,
  AsterizerError
};

pub(crate) trait MakeAst where Self: Sized {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError>;
}
