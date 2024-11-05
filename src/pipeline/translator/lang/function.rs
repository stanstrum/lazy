use ast::Identifier;

use super::*;

/// A function argument, stored separately for organizational purposes
#[allow(unused)]
#[derive(Debug)]
pub(crate) struct FunctionArgument {
  pub(crate) parent: OpaqueParent<WeakCell<Function>>,
  /// The name of this argument
  pub(crate) name: ast::Identifier<DefaultWorkflow>,
  /// The type of this argument
  pub(crate) ty: RcCell<Type<Module>>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Variable {
  pub(crate) name: Identifier<DefaultWorkflow>,
  pub(crate) ty: RcCell<Type<Module>>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct FunctionBlock {
  pub(crate) parent: OpaqueParent<Option<WeakCell<Function>>>,
  pub(crate) variables: Vec<RcCell<Variable>>,
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
pub(crate) struct Function {
  pub(crate) parent: OpaqueParent<Option<WeakCell<Module>>>,
  pub(crate) name: ast::Identifier<DefaultWorkflow>,
  pub(crate) arguments: Vec<RcCell<FunctionArgument>>,
  pub(crate) body: RcCell<FunctionBlock>,
  pub(crate) return_ty: RcCell<Type<Module>>,
}
