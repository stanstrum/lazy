use crate::lang::module::ModuleId;
use crate::string_pool::{CommentId, PoolId};
use crate::aster::bufreader::Metadata;

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
pub struct Span {
  pub start: Position,
  pub end: Position,
  pub module: ModuleId,
}

pub type TokenSpan = (Token, Span);

#[derive(Debug, Clone, Copy)]
pub enum Token {
  Identifier(PoolId),
  Keyword(Keyword),
  Operator(Operator),
  Grouping(GroupingType),
  Whitespace,
  Indent(isize),
  Comment(CommentId),
  Numeric(NumericValue),
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
});

#[derive(Debug, Clone, Copy)]
pub enum Operator {
  RightArrow,
  Range,
  DoubleColon,
}

/// Contains only the start position of a Span
#[derive(Debug, Clone, Copy)]
pub struct Position {
  pub position: usize,
  pub line: usize,
  pub column: usize,
  pub indentation: usize,
}

impl Position {
  pub fn new() -> Self {
    Self {
      position: 0,
      line: 1,
      column: 1,
      indentation: 0,
    }
  }

  pub fn new_from_meta(meta: &Metadata) -> Self {
    Self {
      position: meta.position,
      line: meta.line,
      column: meta.column,
      indentation: meta.whitespace,
    }
  }
}

impl Span {
  pub fn from_pair(module: ModuleId, start: Span, end: Span) -> Self {
    Self {
      start: start.start,
      end: end.end,
      module,
    }
  }

  pub fn extend(&mut self, other: Span) {
    assert!(self.module == other.module);
    self.end = other.end;
  }
}
