use crate::asterizer::ast::*;
use typename::TypeName;

/// A standard function argument, i.e. an identifier and a simple type
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct FunctionArgument {
  /// The name of the argument
  pub(crate) identifier: Identifier,
  /// The type of the argument
  pub(crate) ty: Type,
  pub(crate) span: Span,
}

/// The arguments to a function
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct FunctionArguments {
  /// The arguments of this function
  pub(crate) arguments: Vec<FunctionArgument>,
}

/// A simple function, i.e. not a class method, meaning no "this" reference can
/// be held here
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Function {
  /// The name of this function
  pub(crate) identifier: Identifier,
  /// The return type of this function -- optional, defaults to void
  pub(crate) return_ty: Option<Type>,
  /// The arguments of this function -- optional
  pub(crate) arguments: Option<FunctionArguments>,
  /// The body of this function
  pub(crate) body: BlockExpression,
  pub(crate) span: Span,
}
