use std::path::PathBuf;

use crate::lang::reference::{ModuleReference, FunctionReference};
use crate::lang::ty::Type;
use crate::string_pool::PoolId;
use crate::tokenize::token::Span;

#[derive(Debug, Clone, Copy)]
pub struct TokensId(pub usize);

#[derive(Debug)]
pub struct ModulePath {
  pub path: PathBuf,
  pub tokens: TokensId,
}

#[derive(Debug)]
pub enum ModuleParent {
  Path(ModulePath),
  Module(ModuleReference),
}

#[derive(Debug)]
pub struct Module {
  pub name: PoolId,
  pub modules: Vec<ModuleReference>,
  pub functions: Vec<FunctionReference>,
  pub parent: ModuleParent,
  pub aliases: Vec<TypeAlias>,
}


#[derive(Debug)]
pub struct TypeAlias {
  pub name: Name,
  pub ty: Type,
  pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct Name {
  pub id: PoolId,
  pub span: Span,
}

impl Module {
  pub fn new(name: PoolId, parent: ModuleParent) -> Self {
    Self {
      name,
      parent,
      modules: vec![],
      functions: vec![],
      aliases: vec![],
    }
  }
}
