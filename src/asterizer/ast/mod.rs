mod function;
mod expression;
mod impls;

use typename::TypeName;
use crate::tokenizer::Span;

use function::*;
use expression::*;

/// A simple name, this is equivalent to a String but associated with a Span
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Identifier {
  /// The text of this identifier
  pub(crate) name: String,
  pub(crate) span: Span,
}

/// A simple type, i.e. non-arithmetic
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum Type {
  /// A type that is only referred to by name
  Identifier(Identifier),
}

/// A structure that can appear inside of a namespace
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum NamespaceChild {
  Namespace(Box<Namespace>),
  Function(Function),
}

/// A namespace, akin to a module, however modules can only be used to organize
/// code inside of a file.  Inside of a file, namespaces are used to accomplish
/// this.
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Namespace {
  /// The name of this namespace
  pub(crate) identifier: Identifier,
  /// The structures inside this namespace
  pub(crate) children: NamespaceChild,
  pub(crate) span: Span,
}

/// The top-level namespace of the module.  Import statements can only appear
/// here and exports allow visibility outside of the module.
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct TopLevelNamespace {
  /// The structures in this file
  pub(crate) children: Vec<NamespaceChild>,
  pub(crate) span: Span,
}
