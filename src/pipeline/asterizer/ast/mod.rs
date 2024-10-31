mod function;
mod expression;
mod impls;

use typename::TypeName;
use crate::tokenizer::Span;
use crate::compiler::CompilerWorkflow;

pub(crate) use function::*;
pub(crate) use expression::*;

/// A simple name, this is equivalent to a String but associated with a Span
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Identifier<W: CompilerWorkflow> {
  /// The text of this identifier
  pub(crate) name: String,
  pub(crate) span: Span<W>,
}

/// A qualified identifier is an identifier with one or more concatenated parts
/// using double colons (::):
///
/// ```
/// namespace Test {
///   export type Something := usize;
/// };
///
/// do_something {
///   value: Test::Something = 5;
/// };
/// ```
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Qualified<W: CompilerWorkflow> {
  pub(crate) implicit: bool,
  /// The parts of this qualified identifier
  pub(crate) parts: Vec<Identifier<W>>,
  pub(crate) span: Span<W>,
}

/// A simple type, i.e. non-arithmetic
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum Type<W: CompilerWorkflow> {
  /// A type that is only referred to by name
  Qualified(Qualified<W>),
}

/// A structure that can appear inside of a namespace
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum NamespaceChild<W: CompilerWorkflow> {
  Namespace(Box<Namespace<W>>),
  Function(Function<W>),
}

/// A namespace, akin to a module, however modules can only be used to organize
/// code inside of a file.  Inside of a file, namespaces are used to accomplish
/// this.
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Namespace<W: CompilerWorkflow> {
  /// The name of this namespace
  pub(crate) identifier: Identifier<W>,
  /// The structures inside this namespace
  pub(crate) children: Vec<NamespaceChild<W>>,
  pub(crate) span: Span<W>,
}

/// The top-level namespace of the module.  Import statements can only appear
/// here and exports allow visibility outside of the module.
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct TopLevelNamespace<W: CompilerWorkflow> {
  /// The structures in this file
  pub(crate) children: Vec<NamespaceChild<W>>,
  pub(crate) span: Span<W>,
}
