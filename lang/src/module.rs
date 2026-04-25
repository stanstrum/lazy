use std::path::PathBuf;
use std::collections::HashMap;

use string_pool::PoolId;

use crate::Compiler;
use crate::expr::Variable;
use crate::reference::{Store, TypePartId, TypePartReference};
use crate::ty::{Qualified, QualifiedSearchSpace, Type};
use crate::span::Span;

#[derive(Debug, Clone, Copy)]
pub struct Name<C: Compiler> {
  pub id: PoolId,
  pub span: Span<C>,
}

#[derive(Debug)]
pub struct ModulePath<C: Compiler> {
  pub path: PathBuf,
  pub tokens: C::TokensReference,
  pub module: C::ModuleReference,
}

#[derive(Debug)]
pub enum ModuleParent<C: Compiler> {
  Path(ModulePath<C>),
  Module(C::ModuleReference),
}

#[derive(Debug)]
pub struct ModuleTransports<C: Compiler> {
  pub import_map: HashMap<PoolId, Qualified<C>>,
  pub import_stars: Vec<(QualifiedSearchSpace<C>, Span<C>)>,
}

#[derive(Debug)]
pub struct Module<C: Compiler> {
  pub name: PoolId,
  pub transports: ModuleTransports<C>,
  pub modules: Vec<C::ModuleReference>,
  pub functions: Vec<C::FunctionReference>,
  pub parent: ModuleParent<C>,
  pub aliases: Vec<TypeAlias<C>>,
  pub structs: Vec<Struct<C>>,
  pub type_parts: Vec<Type<C>>,
}

#[derive(Debug)]
pub struct TypeAlias<C: Compiler> {
  pub name: Name<C>,
  pub ty: Type<C>,
  pub span: Span<C>,
}

#[derive(Debug)]
pub struct Struct<C: Compiler> {
  pub name: Name<C>,
  pub members: Vec<Variable<C>>,
  pub span: Span<C>,
}

impl<C: Compiler> Module<C> {
  pub fn new(name: PoolId, parent: ModuleParent<C>) -> Self {
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

pub trait AddTypePart<C: Compiler>: Sized where for<'a> C::Store<'a>: Store<Self, Out = crate::module::Module<C>> {
  fn add_type_part(&self, part: crate::ty::Type<C>, store: &mut C::Store<'_>) -> TypePartReference<C>;
}

impl<C: Compiler> AddTypePart<C> for C::ModuleReference {
  fn add_type_part(&self, part: crate::ty::Type<C>, store: &mut <C as Compiler>::Store<'_>) -> TypePartReference<C> {
    let module_ref = store.rget_mut(*self);

    let id = TypePartId(module_ref.type_parts.len());
    module_ref.type_parts.push(part);

    TypePartReference::<C>(*self, id)
  }
}
