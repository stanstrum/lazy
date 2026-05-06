use string_pool::{PoolId, StringId};

use crate::span::Span;
use crate::reference::ExpressionReference;
use crate::Compiler;

pub type TokenSpan<C> = (Token, Span<C>);
pub type Tokens<C> = Vec<TokenSpan<C>>;

#[derive(Debug, Clone, Copy)]
pub enum Token {
  Identifier(PoolId),
  Keyword(Keyword),
  Operator(Operator),
  Grouping(GroupingType),
  Whitespace,
  Indent(isize),
  Comment(StringId),
  Numeric(NumericValue),
  String(StringKind, StringId)
}

#[derive(Debug, Clone, Copy)]
pub enum StringKind {
  Wide,
  Byte,
  C,
}

#[derive(Debug, Clone, Copy)]
pub enum CharKind {
  Wide,
  Byte,
}

macro_rules! string_enum {
  ($name:ident { $($entries:ident => $values:expr,)* }) => {
  #[derive(Debug, Clone, Copy)]
  pub enum $name {
      $($entries,)*
    }

    impl $name {
      pub fn from_str(str: &str) -> Option<Self> {
        match str {
          $($values => Some(Self::$entries),)*
          _ => None,
        }
      }
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
          $(Self::$entries => $values.into(),)*
        })
      }
    }
  };
}

#[derive(Debug, Clone, Copy)]
pub enum NumericKind {
  Binary,
  Ternary,
  Seximal,
  Octal,
  Decimal,
  Hexadecimal,
  Roman,
}

#[derive(Debug, Clone, Copy)]
pub enum NumericValue {
  U64(u64),
  F64(f64),
}

#[derive(Debug, Clone, Copy)]
pub enum GroupingType {
  Open(GroupingKind),
  Close(GroupingKind),
}

#[derive(Debug, Clone, Copy)]
pub enum GroupingKind {
  Parenthesis,
  Bracket,
  Brace,
}

string_enum!(Keyword {
  Import => "import",
  Export => "export",
  From => "from",
  As => "as",
  Do => "do",
  While => "while",
  For => "for",
  Until => "until",
  In => "in",
  Return => "return",
  Break => "break",
  Continue => "continue",
  Yield => "yield",
  If => "if",
  Else => "else",
  Switch => "switch",
  Match => "match",
  Case => "case",
  Mod => "mod",
  Struct => "struct",
  Interface => "interface",
  Abstract => "abstract",
  Class => "class",
  Private => "private",
  Protected => "protected",
  Public => "public",
  Template => "template",
  Extends => "extends",
  Infer => "infer",
  Type => "type",
  Mut => "mut",
});

#[derive(Debug, Clone, Copy)]
pub enum Operator {
  Plus,
  Minus,
  Asterisk,
  Div,
  Mod,
  AddAssign,
  SubAssign,
  MulAssign,
  ExpAssign,
  DivAssign,
  ModAssign,

  Or,
  SingleAnd,
  Xor,
  Shl,
  Shr,
  OrAssign,
  AndAssign,
  XorAssign,
  ShlAssign,
  ShrAssign,
  LogicalOr,
  LogicalAnd,
  LogicalXor,
  LogicalShr,
  LogicalOrAssign,
  LogicalAndAssign,
  LogicalXorAssign,
  LogicalShrAssign,

  Not,
  Invert,

  Greater,
  GreaterEqual,
  Less,
  LessEqual,
  Equal,
  Assign,

  Dot,
  Range,
  Splat,
  // TODO: these are actually punctuation, perhaps break them out into their
  //       own subvariant
  RightArrow,
  DoubleColon,
  Semicolon,
  Colon,
  Bollocks,
  Comma,

  DoublePlus,
  DoubleMinus,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
  Add, // +
  Sub, // -
  Mul, // *
  Div, // /
  Mod, // %
  Exp, // **
  And, // &
  Or, // |
  Xor, // ^
  Shr, // >>
  Shl, // <<
  LogicalAnd, // &&
  LogicalOr, // ||
  LogicalXor, // ^^
  LogicalShr, // >>>
  DerefDot, // ->
  Dot, // .

  Assign, // =
  AddAssign, // +=
  SubAssign, // -=
  MulAssign, // *=
  DivAssign, // /=
  ModAssign, // %=
  ExpAssign, // **=
  AndAssign, // &=
  OrAssign, // |=
  XorAssign, // ^=
  ShlAssign, // >>=
  ShrAssign, // <<=
  LogicalAndAssign, // &&=
  LogicalOrAssign, // ||=
  LogicalXorAssign, // ^^=
  LogicalShrAssign, // >>>=

  Less, // <
  LessEqual, // <=
  Greater, // >
  GreaterEqual, // >=
  Equal, // ==

  Fish, // <>
  Range, // ..
  Splat, // ...
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryPrefixOperator {
  Deref,
  Ref,
  MutRef,
  Not,
  Invert,
  Identity,
  Negate,
  PreDecrement,
  PreIncrement,
  Splat,
}

#[derive(Debug, Clone)]
pub enum UnarySuffixOperator<C: Compiler> {
  Try,
  Call(Vec<ExpressionReference<C>>),
  PostDecrement,
  PostIncrement,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator<C: Compiler> {
  Prefix(UnaryPrefixOperator),
  Suffix(UnarySuffixOperator<C>),
}

impl std::fmt::Display for BinaryOperator {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      BinaryOperator::Add => "+",
      BinaryOperator::Sub => "-",
      BinaryOperator::Mul => "*",
      BinaryOperator::Div => "/",
      BinaryOperator::Mod => "%",
      BinaryOperator::Exp => "**",
      BinaryOperator::And => "&",
      BinaryOperator::Or => "|",
      BinaryOperator::Xor => "^",
      BinaryOperator::Shr => ">>",
      BinaryOperator::Shl => "<<",
      BinaryOperator::LogicalAnd => "&&",
      BinaryOperator::LogicalOr => "||",
      BinaryOperator::LogicalXor => "^^",
      BinaryOperator::LogicalShr => ">>>",
      BinaryOperator::Dot => ".",
      BinaryOperator::DerefDot => "->",
      BinaryOperator::Assign => "=",
      BinaryOperator::AddAssign => "+=",
      BinaryOperator::SubAssign => "-=",
      BinaryOperator::MulAssign => "*=",
      BinaryOperator::DivAssign => "/=",
      BinaryOperator::ModAssign => "%=",
      BinaryOperator::ExpAssign => "**=",
      BinaryOperator::AndAssign => "&=",
      BinaryOperator::OrAssign => "|=",
      BinaryOperator::XorAssign => "^=",
      BinaryOperator::ShlAssign => "<<=",
      BinaryOperator::ShrAssign => ">>=",
      BinaryOperator::LogicalAndAssign => "&&=",
      BinaryOperator::LogicalOrAssign => "||=",
      BinaryOperator::LogicalXorAssign => "^^=",
      BinaryOperator::LogicalShrAssign => ">>>=",
      BinaryOperator::Less => "<",
      BinaryOperator::LessEqual => "<=",
      BinaryOperator::Greater => ">",
      BinaryOperator::GreaterEqual => ">=",
      BinaryOperator::Equal => "==",
      BinaryOperator::Fish => "<>",
      BinaryOperator::Range => "..",
      BinaryOperator::Splat => "...",
    })
  }
}

impl From<StringKind> for crate::intrinsic::Intrinsic {
  fn from(value: StringKind) -> Self {
    match value {
      StringKind::Wide => Self::U32,
      StringKind::Byte | StringKind::C => Self::U8,
    }
  }
}
