use crate::tokenizer::Span;

mod impls;

/// A simple type, i.e. non-arithmetic
pub(crate) enum Type {
  /// A type that is only referred to by name
  Identifier(Identifier),
}

/// A simple name, this is equivalent to a String but associated with a Span
pub(crate) struct Identifier {
  /// The text of this identifier
  pub(crate) name: String,
  pub(crate) span: Span,
}

/// A standard function argument, i.e. an identifier and a simple type
pub(crate) struct FunctionArgument {
  /// The name of the argument
  pub(crate) identifier: Identifier,
  /// The type of the argument
  pub(crate) ty: Type,
  pub(crate) span: Span,
}

/// A simple function, i.e. not a class method, meaning no "this" reference can
/// be held here
pub(crate) struct Function {
  /// The name of this function
  pub(crate) identifier: Identifier,
  /// The return type of this function -- optional, defaults to void
  pub(crate) return_ty: Option<Type>,
  /// The arguments of this function
  pub(crate) arguments: Vec<FunctionArgument>,
  pub(crate) span: Span,
}

/// A structure that can appear inside of a namespace
pub(crate) enum NamespaceChild {
  Namespace(Box<Namespace>),
  Function(Function),
}

/// A namespace, akin to a module, however modules can only be used to organize
/// code inside of a file.  Inside of a file, namespaces are used to accomplish
/// this.
pub(crate) struct Namespace {
  /// The name of this namespace
  pub(crate) identifier: Identifier,
  /// The structures inside this namespace
  pub(crate) children: NamespaceChild,
  pub(crate) span: Span,
}

/// The top-level namespace of the module.  Import statements can only appear
/// here and exports allow visibility outside of the module.
pub(crate) struct TopLevelNamespace {
  /// The structures in this file
  pub(crate) children: Vec<NamespaceChild>,
  pub(crate) span: Span,
}
