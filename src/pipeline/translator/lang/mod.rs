mod r#type;
mod function;
mod reference;

pub(crate) use r#type::*;
pub(crate) use function::*;
pub(crate) use reference::*;

use std::fmt::Debug;

use crate::compiler::{
  CompilerStoreHandle,
  workflow::DefaultWorkflow,
  CompilerWorkflow,
};

use crate::asterizer::ast;
use crate::tokenizer::Span;

#[derive(Clone)]
pub(crate) struct OpaqueParent<T: Clone> {
  parent: T,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct TypeAlias {
  pub(crate) parent: OpaqueParent<WeakCell<Module>>,
  pub(crate) name: ast::Identifier<DefaultWorkflow>,
  pub(crate) ty: RcCell<Type<Module>>,
}

/// A member/child of a Module
#[allow(unused)]
#[derive(Debug)]
pub(crate) enum ModuleChild {
  Function(RcCell<Function>),
  Module(RcCell<Module>),
  Type(RcCell<TypeAlias>),
}

/// The name of a module.  This is a variant because top-level namespaces are
/// treated the same as namespaces in files, however these top-level namespaces
/// have no written name, so we store the Handle instead
#[allow(unused)]
#[derive(Debug)]
pub(crate) enum ModuleName<W: CompilerWorkflow = DefaultWorkflow> {
  /// This module has a name at a specific Span
  Identifier(ast::Identifier<W>),
  /// This module is a file's top-level namespace
  File(CompilerStoreHandle<W>),
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Import {
  pub(crate) name: String,
  pub(crate) reference: RcCell<Reference<ModuleChild, Module>>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Export {
  pub(crate) name: Option<String>,
  pub(crate) reference: RcCell<Reference<ModuleChild, Module>>,
}

/// Represents a Module, specifically, the top-level namespace thereof
#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Module {
  pub(crate) parent: OpaqueParent<Option<WeakCell<Module>>>,
  /// The name of this module
  pub(crate) name: ModuleName,
  /// The children/members of this module
  pub(crate) children: Vec<RcCell<ModuleChild>>,
  pub(crate) span: Span<DefaultWorkflow>,
  // TODO: imports, exports, ...
  pub(crate) imports: Vec<Import>,
  pub(crate) exports: Vec<Export>,
  pub(crate) generator_id: Option<usize>,
}

impl<T: Clone> From<T> for OpaqueParent<T> {
  fn from(parent: T) -> Self {
    Self { parent }
  }
}

impl<T: Clone> OpaqueParent<T> {
  pub(crate) fn unwrap(self) -> T {
    self.parent
  }

  pub(crate) fn as_ref(&self) -> &T {
    &self.parent
  }
}

impl PartialEq<&str> for ModuleName {
  fn eq(&self, other: &&str) -> bool {
    match self {
      ModuleName::Identifier(identifier) => &identifier.name == other,
      ModuleName::File(_) => {
        warn!("maybe duplicating a file?");

        false
      }
    }
  }
}

impl<T: Clone> Debug for OpaqueParent<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("(parent)")
  }
}
