use typename::TypeName;

use crate::compiler::CompilerWorkflow;
use crate::tokenizer::Span;

#[allow(unused)]
/// Represents a String literal of any kind
#[derive(Debug, TypeName)]
pub(crate) enum StringLiteral {
  /// An unadorned String, which can be coerced to the following types:
  ///
  /// ```
  /// // A global slice of chars
  /// value: &'static []char := "hello";
  /// // A global reference to a const-sized array of chars
  /// value: &'static [5]char := "hello";
  /// // A stack-allocated const-sized array of chars
  /// value: [5]char := "hello";
  /// ```
  Generic(String),
  /// An null-terminated "C" String.  Note that this string serializes to a
  /// sequence of u8 bytes, not chars.
  ///
  /// ```
  /// // A global slice of null-delimited bytes
  /// value: &'static []u8 := c"hello";
  /// // A global reference to a const-sized array of null-delimited ASCII
  /// value: &'static [N]char := "hello";
  /// // A stack-allocated const-size array off null-delimited ASCII
  /// value: [6]u8 := c"hello";
  /// ```
  CString(String),
}

#[allow(unused)]
/// Represents a Char literal of any kind
#[derive(Debug, TypeName)]
pub(crate) enum CharLiteral {
  /// An unadorned Char, which is implictly equivalent to a unicode char
  Generic(String),
  /// A "C" Char, which is implicitly equivalent to an ASCII byte (u8)
  Byte(u8),
}

#[allow(unused)]
/// Represents a Numeric literal of any kind
#[derive(Debug, TypeName)]
pub(crate) enum NumericLiteral {
  /// An unadorned Numeric, which can be coerced as follows:
  ///
  /// ```
  /// // From i8 ...
  /// value: i8 := 255;
  /// // ... all the way up to u64
  /// value: u64 := 255;
  /// // Or as a float
  /// value: f64 := 255;
  /// ```
  Generic(u64),
  /// A floating-point Numeric, which can be coerced to size-applicable float
  /// types
  Float(f64),
}

#[allow(unused)]
/// A Literal of any kind
#[derive(Debug, TypeName)]
pub(crate) enum LiteralKind {
  /// A String literal
  String(StringLiteral),
  /// A Char literal
  Char(CharLiteral),
  /// A Numeric literal
  Numeric(NumericLiteral),
}

#[allow(unused)]
/// A Literal with an associated span for debugging
#[derive(Debug, TypeName)]
pub(crate) struct Literal<W: CompilerWorkflow> {
  pub(crate) kind: LiteralKind,
  pub(crate) span: Span<W>,
}
