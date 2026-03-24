use std::path::PathBuf;

use crate::lang::Lazy;
use crate::lang::reference::{FunctionReference, ModuleReference, Reference, TypePartReference};
use crate::lang::ty::Type;
use crate::string_pool::PoolId;
use crate::tokenize::token::Span;

#[derive(Debug, Clone, Copy)]
pub struct TokensId(pub usize);

#[derive(Debug)]
pub struct ModulePath {
  pub path: PathBuf,
  pub tokens: TokensId,
  pub module: ModuleReference,
}

#[derive(Debug)]
pub enum ModuleParent {
  Path(ModulePath),
  Module(ModuleReference),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypePartId(pub usize);

#[derive(Debug)]
pub struct Module {
  pub name: PoolId,
  pub modules: Vec<ModuleReference>,
  pub functions: Vec<FunctionReference>,
  pub parent: ModuleParent,
  pub aliases: Vec<TypeAlias>,
  pub type_parts: Vec<Type>,
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
      type_parts: vec![],
    }
  }
}

impl ModuleReference {
  pub fn add_type_part(&self, part: Type, lazy: &mut Lazy) -> TypePartReference {
    let module_ref = self.rget_from_mut(lazy);

    let id = TypePartId(module_ref.type_parts.len());
    module_ref.type_parts.push(part);

    TypePartReference(*self, id)
  }
}
