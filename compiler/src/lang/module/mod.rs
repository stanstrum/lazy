pub mod import;
pub mod struc;

use crate::Lazy;
use crate::lang::reference::{ModuleReference, Reference, TypePartReference};
use crate::lang::ty::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokensId(pub usize);

pub type ModulePath = ::lang::module::ModulePath<crate::lazy::LazyStructures>;
pub type ModuleParent = ::lang::module::ModuleParent<crate::lazy::LazyStructures>;
// pub type ModuleTransports = ::lang::module::ModuleTransports<crate::lazy::LazyStructures>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypePartId(pub usize);

pub type Module = ::lang::module::Module<crate::lazy::LazyStructures>;
pub type TypeAlias = ::lang::module::TypeAlias<crate::lazy::LazyStructures>;
pub type Name = ::lang::module::Name<crate::lazy::LazyStructures>;

impl ModuleReference {
  pub fn add_type_part(&self, part: Type, lazy: &mut Lazy) -> TypePartReference {
    let module_ref = self.rget_from_mut(lazy);

    let id = TypePartId(module_ref.type_parts.len());
    module_ref.type_parts.push(part);

    TypePartReference(*self, id)
  }
}
