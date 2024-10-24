use crate::asterizer::ast::*;
use typename::TypeName;

/// A standard function argument, i.e. an identifier and a simple type
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct FunctionArgument<W: CompilerWorkflow> {
  /// The name of the argument
  pub(crate) identifier: Identifier<W>,
  /// The type of the argument
  pub(crate) ty: Type<W>,
  pub(crate) span: Span<W>,
}

/// The arguments to a function
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct FunctionArguments<W: CompilerWorkflow> {
  /// The arguments of this function
  pub(crate) arguments: Vec<FunctionArgument<W>>,
  pub(crate) span: Span<W>,
}

/// A simple function, i.e. not a class method, meaning no "this" reference can
/// be held here
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Function<W: CompilerWorkflow> {
  /// The name of this function
  pub(crate) identifier: Identifier<W>,
  /// The return type of this function -- optional, defaults to void
  pub(crate) return_ty: Option<Type<W>>,
  /// The arguments of this function -- optional
  pub(crate) arguments: Option<FunctionArguments<W>>,
  /// The body of this function
  pub(crate) body: BlockExpression<W>,
  pub(crate) span: Span<W>,
}
