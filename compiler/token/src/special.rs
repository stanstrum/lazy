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
