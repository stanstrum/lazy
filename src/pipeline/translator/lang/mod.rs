mod r#type;
mod function;
pub(crate) use r#type::*;
pub(crate) use function::*;

use crate::compiler::{
  CompilerStoreHandle,
  workflow::DefaultWorkflow,
  CompilerWorkflow,
};

use crate::asterizer::ast;
use crate::tokenizer::Span;

/// A member/child of a Module
#[allow(unused)]
#[derive(Debug)]
pub(crate) enum ModuleChild<W: CompilerWorkflow> {
  Function(Function<W>),
  Module(Box<Module<W>>),
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
pub(crate) struct Module<W: CompilerWorkflow> {
  /// The name of this module
  pub(crate) name: ModuleName<W>,
  /// The children/members of this module
  pub(crate) children: Vec<ModuleChild<W>>,
  pub(crate) span: Span<W>,
  // TODO: imports, exports, ...
}
