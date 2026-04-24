use std::path::PathBuf;
use std::collections::HashMap;

use string_pool::PoolId;

use crate::Compiler;
use crate::expr::Variable;
use crate::ty::{Qualified, QualifiedSearchSpace, Type};
use crate::span::Span;

#[derive(Debug)]
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
  pub aliases: Vec<C::TypeAlias>,
  pub structs: Vec<C::Struct>,
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

impl<C: Compiler> Clone for Name<C> {
  fn clone(&self) -> Self {
    Self {
      id: self.id.clone(),
      span: self.span.clone(),
    }
  }
}

impl<C: Compiler> Copy for Name<C> {}

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
