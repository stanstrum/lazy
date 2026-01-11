use crate::lang::ModuleId;
use crate::string_pool::PoolId;
use crate::bufreader::Metadata;

macro_rules! string_enum {
  ($name:ident { $($entries:ident => $values:expr,)* }) => {
    #[derive(Debug)]
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

    impl std::string::ToString for $name {
      fn to_string(&self) -> String {
        match self {
          $(Self::$entries => $values.into(),)*
        }
      }
    }
  };
}

#[derive(Debug)]
pub struct TokenSpan {
  pub tok: Token,
  pub start: Position,
  pub end: Position,
  pub module: ModuleId,
}

#[derive(Debug)]
pub enum Token {
  Identifier(PoolId),
  Keyword(Keyword),
  Operator(Operator),
  Grouping(GroupingType),
  Whitespace,
  Indent(isize),
  Comment(String),
}

#[derive(Debug)]
pub enum GroupingType {
  Open(GroupingKind),
  Close(GroupingKind),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Operator {
  RightArrow,
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
  pub fn new(meta: &Metadata) -> Self {
    Self {
      position: meta.position,
      line: meta.line,
      column: meta.column,
      indentation: meta.whitespace,
    }
  }
}
