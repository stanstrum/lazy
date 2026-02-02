use std::path::PathBuf;

use crate::lang::ty::Type;
use crate::string_pool::PoolId;
use crate::tokenize::token::Span;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModuleId(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct TokensId(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct FunctionId(pub usize);

#[derive(Debug)]
pub struct ModulePath {
  pub path: PathBuf,
  pub tokens: TokensId,
}

#[derive(Debug)]
pub enum ModuleParent {
  Path(ModulePath),
  Module(ModuleId),
}

#[derive(Debug)]
pub struct Module {
  pub name: PoolId,
  pub modules: Vec<ModuleId>,
  pub functions: Vec<FunctionId>,
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
