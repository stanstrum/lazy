macro_rules! enum_map {
  ($name:ident { $($ident:ident: $expr:expr),+, }) => {
    $(
      #[allow(non_upper_case_globals)]
      const $ident: &str = $expr;
    )+

    #[derive(Debug)]
    pub(crate) enum $name {
      $($ident,)+
    }

    #[allow(unused)]
    impl $name {
      #[allow(non_upper_case_globals)]
      pub(crate) fn from_str(text: &str) -> Option<Self> {
        match text {
          $($ident => Some(Self::$ident),)+
          _ => None,
        }
      }

      pub(crate) fn to_str(&self) -> &'static str {
        match self {
          $(Self::$ident => $ident,)+
        }
      }
    }
  };
}

enum_map!(Operator {
  BitNot: "~",
  LogicalNot: "!",
  Modulo: "%",
  ModuloAssign: "%=",
  BitXor: "^",
  BitXorAssign: "^=",
  LogicalXor: "^^",
  LogicalXorAssign: "^^=",
  Ampersand: "&",
  BitAndAssign: "&=",
  LogicalAnd: "&&",
  LogicalAndAssign: "&&=",
  Asterisk: "*",
  MulAssign: "*=",
  Exp: "**",
  ExpAssign: "**=",
  Minus: "-",
  SubAssign: "-=",
  Decrement: "--",
  Plus: "+",
  AddAssign: "+=",
  Increment: "++",
  Assign: "=",
  Equals: "==",
  BitOr: "|",
  BitOrAssign: "|=",
  LogicalOr: "||",
  LogicalOrAssign: "||=",
  LessThan: "<",
  LessThanEquals: "<=",
  BitShiftLeft: "<<",
  BitShiftLeftAssign: "<<=",
  GreaterThan: ">",
  GreaterThanEquals: ">=",
  BitShiftRight: ">>",
  BitShiftRightAssign: ">>=",
  LogicalShiftRight: ">>>",
  LogicalShiftRightAssign: ">>>=",
  Dot: ".",
  Div: "/",
  DivAssign: "/=",
  Try: "?",
});

enum_map!(Keyword {
  Continue: "continue",
  Break: "break",
  Return: "return",
  Template: "template",
  Extends: "extends",
  Implements: "implements",
  Satisfies: "satisfies",
  Infer: "infer",
  Type: "type",
  Const: "const",
  Mut: "mut",
  Struct: "struct",
  Interface: "interface",
  Class: "class",
  Namespace: "namespace",
  Public: "public",
  Protected: "protected",
  Private: "private",
  Static: "static",
  Abstract: "abstract",
  Import: "import",
  Export: "export",
  From: "from",
  As: "as",
  If: "if",
  Else: "else",
  Switch: "switch",
  Match: "match",
  For: "for",
  Loop: "loop",
  Unless: "unless",
  Do: "do",
  While: "while",
});

enum_map!(Punctuation {
  Colon: ":",
  Semicolon: ";",
  Comma: ",",
  Ellipsis: "...",
  RightArrow: "->",
  DoubleColon: "::",
  Bollocks: ":=",
});

enum_map!(Grouping {
  OpenParenthesis: "(",
  OpenBracket: "[",
  OpenBrace: "{",
  CloseParenthesis: ")",
  CloseBracket: "]",
  CloseBrace: "}",
});
