mod impls;

use typename::TypeName;
use crate::tokenizer::Span;

/// A simple type, i.e. non-arithmetic
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum Type {
  /// A type that is only referred to by name
  Identifier(Identifier),
}

/// A simple name, this is equivalent to a String but associated with a Span
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Identifier {
  /// The text of this identifier
  pub(crate) name: String,
  pub(crate) span: Span,
}

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

/// An expression of any kind
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum Expression {
  Block(Box<BlockExpression>),
}

/// A variable binding, which has either a type, a bound expression, or both --
/// however a binding may not have neither as it would conflict with the syntax
/// of simply recalling the value of a variable, e.g.:
///
/// With type:
/// ```
/// foo: bool;
/// ```
///
/// With expression:
/// ```
/// bar = 0u32;
/// ```
///
/// With both:
/// ```
/// foo_bar: f32 = 1.0;
/// ```
///
/// However, having neither would (hypothetically) read as follows:
/// ```
/// bad_variable;
/// ```
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum BindingKind {
  OnlyType(Type),
  OnlyExpression(Expression),
  Both {
    ty: Type,
    expression: Expression,
  }
}

/// A variable binding with the identifier
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Binding {
  /// The name of this variable
  pub(crate) identifier: Identifier,
  /// The specifying information of this variable
  pub(crate) kind: BindingKind,
}

/// A child of a function block
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum BlockChild {
  Binding(Binding),
}

/// A function block, with curly braces at the beginning and end
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct BlockExpression {
  /// The expressions inside of this block
  #[allow(unused)]
  pub(crate) children: Vec<BlockChild>,
  /// If this block uses shorthand to return the value of the last statement,
  /// then it will appear here.  Note that this value is of type Expression
  /// rather than BlockChild -- this is because bindings yield no value and
  /// therefore cannot be returned.
  ///
  /// Example:
  /// ```
  /// main -> i32 {
  ///   0
  /// };
  /// ```
  ///
  /// as opposed to:
  /// ```
  /// main -> i32 {
  ///   return 0;
  /// };
  /// ```
  pub(crate) return_last: Option<Expression>,
  pub(crate) span: Span,
}
