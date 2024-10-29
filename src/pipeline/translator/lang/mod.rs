mod r#type;
mod function;
pub(crate) use r#type::*;
pub(crate) use function::*;

use crate::compiler::CompilerWorkflow;

use crate::asterizer::ast;
use crate::tokenizer::Span;

/// A member/child of a Module
#[allow(unused)]
#[derive(Debug)]
pub(crate) enum ModuleChild<W: CompilerWorkflow> {
  Function(Function<W>),
  Module(Box<Module<W>>),
}

/// Represents a Module, specifically, the top-level namespace thereof
#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Module<W: CompilerWorkflow> {
  /// The children/members of this module
  pub(crate) children: Vec<ModuleChild<W>>,
  pub(crate) span: Span<W>,
  // TODO: imports, exports, ...
}
