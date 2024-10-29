use super::*;

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
