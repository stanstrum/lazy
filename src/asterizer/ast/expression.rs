use typename::TypeName;

use crate::asterizer::ast::*;

/// An expression of any kind
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum Expression<W: CompilerWorkflow> {
  Block(Box<BlockExpression<W>>),
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
pub(crate) enum BindingKind<W: CompilerWorkflow> {
  OnlyType(Type<W>),
  OnlyExpression(Expression<W>),
  Both {
    ty: Type<W>,
    expression: Expression<W>,
  }
}

/// A variable binding with the identifier
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Binding<W: CompilerWorkflow> {
  /// The name of this variable
  pub(crate) identifier: Identifier<W>,
  /// The specifying information of this variable
  pub(crate) kind: BindingKind<W>,
}

/// A child of a function block
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum BlockChild<W: CompilerWorkflow> {
  Binding(Binding<W>),
}

/// A function block, with curly braces at the beginning and end
#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct BlockExpression<W: CompilerWorkflow> {
  /// The expressions inside of this block
  #[allow(unused)]
  pub(crate) children: Vec<BlockChild<W>>,
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
  pub(crate) return_last: Option<Expression<W>>,
  pub(crate) span: Span<W>,
}
