pub mod import;
pub mod struc;

use std::collections::HashMap;

use crate::lang::module::struc::Struct;
use string_pool::PoolId;
use crate::Lazy;
use crate::lang::reference::{FunctionReference, ModuleReference, Reference, TypePartReference};
use crate::lang::ty::{Qualified, QualifiedSearchSpace, Type};
use crate::tokenize::token::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokensId(pub usize);

pub type ModulePath = ::lang::module::ModulePath<crate::lazy::LazyStructures>;
pub type ModuleParent = ::lang::module::ModuleParent<crate::lazy::LazyStructures>;

#[derive(Debug)]
pub struct ModuleTransports {
  pub import_map: HashMap<PoolId, Qualified>,
  pub import_stars: Vec<(QualifiedSearchSpace, Span)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypePartId(pub usize);

#[derive(Debug)]
pub struct Module {
  pub name: PoolId,
  pub transports: ModuleTransports,
  pub modules: Vec<ModuleReference>,
  pub functions: Vec<FunctionReference>,
  pub parent: ModuleParent,
  pub aliases: Vec<TypeAlias>,
  pub structs: Vec<Struct>,
  pub type_parts: Vec<Type>,
}

#[derive(Debug)]
pub struct TypeAlias {
  pub name: Name,
  pub ty: Type,
  pub span: Span,
}

pub type Name = ::lang::module::Name<crate::lazy::LazyStructures>;

impl Module {
  pub fn new(name: PoolId, parent: ModuleParent) -> Self {
    Self {
      name,
      parent,
      transports: ModuleTransports {
        import_map: HashMap::new(),
        import_stars: Vec::new(),
      },
      modules: vec![],
      functions: vec![],
      aliases: vec![],
      structs: vec![],
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
