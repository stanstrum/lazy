mod r#type;
pub(crate) use r#type::*;

use crate::compiler::CompilerWorkflow;

use crate::asterizer::ast;
use crate::tokenizer::Span;

/// A function argument, stored separately for organizational purposes
#[allow(unused)]
#[derive(Debug)]
pub(crate) struct FunctionArgument<W: CompilerWorkflow> {
  /// The name of this argument
  pub(crate) name: ast::Identifier<W>,
  /// The type of this argument
  pub(crate) ty: Type<W>,
}

/// A simple function, i.e., one that does not belong to a class or interface.
///
/// Example:
/// ```
/// main -> i32 {
///   0
/// };
/// ```
#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Function<W: CompilerWorkflow> {
  pub(crate) name: ast::Identifier<W>,
  pub(crate) arguments: Vec<FunctionArgument<W>>,
  pub(crate) return_ty: Type<W>,
}

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
