mod r#type;
mod function;
mod reference;
use std::fmt::{Debug, Pointer};

pub(crate) use r#type::*;
pub(crate) use function::*;
pub(crate) use reference::*;

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

impl<T: Clone> Debug for OpaqueParent<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("(parent)")
  }
}

/// A member/child of a Module
#[allow(unused)]
#[derive(Debug)]
pub(crate) enum ModuleChild {
  Function(RcCell<Function>),
  Module(RcCell<Module>),
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

/// Represents a Module, specifically, the top-level namespace thereof
#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Module {
  pub(crate) parent: OpaqueParent<Option<RcCell<Module>>>,
  /// The name of this module
  pub(crate) name: ModuleName,
  /// The children/members of this module
  pub(crate) children: Vec<ModuleChild>,
  pub(crate) span: Span<DefaultWorkflow>,
  // TODO: imports, exports, ...
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
}
