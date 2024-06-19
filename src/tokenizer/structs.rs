use typename::TypeName;

use crate::compiler::Handle;

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Span {
  pub(crate) start: usize,
  pub(crate) end: usize,
  pub(crate) handle: Handle,
}

pub(crate) trait GetSpan {
  fn get_span(&self) -> &Span;
}

#[derive(Debug)]
pub(crate) enum Operator {
  RightArrow,
  BindAssign,
  Equals,
  SingleAnd,
  SingleAndAssign,
  DoubleAnd,
  DoubleAndAssign,
  Increment,
  Add,
  AddAssign,
  Decrement,
  Subtract,
  SubtractAssign,
  Exponent,
  ExponentAssign,
  Multiply,
  MultiplyAssign,
  Modulo,
  ModuloAssign,
  // integer decrement?
  Divide,
  DivideAssign,
  Range,
  Dot,
  Separator,
  ShiftLeft,
  ShiftLeftAssign,
  ShiftRight,
  ShiftRightAssign,
  LogicalShiftRight,
  LogicalShiftRightAssign,
  Equality,
  GreaterThan,
  GreaterThanEqual,
  LessThan,
  LessThanEqual,
}

#[derive(Debug)]
pub(crate) enum Punctuation {
  Colon,
  Comma,
  Semicolon,
  VariadicEllipsis,
}

#[derive(Debug)]
pub(crate) enum CommentType {
  Line,
  Multiline,
}

mod keywords {
  pub(super) const TYPE: &str = "type";
  pub(super) const IMPORT: &str = "import";
  pub(super) const EXPORT: &str = "export";
  pub(super) const FROM: &str = "from";
  pub(super) const STRUCT: &str = "struct";
  pub(super) const CLASS: &str = "class";
  pub(super) const PRIVATE: &str = "private";
  pub(super) const PROTECTED: &str = "protected";
  pub(super) const PUBLIC: &str = "public";
  pub(super) const ABSTRACT: &str = "abstract";
  pub(super) const INTERFACE: &str = "interface";
  pub(super) const NAMESPACE: &str = "namespace";
  pub(super) const IMPLEMENTS: &str = "implements";
  pub(super) const IMPL: &str = "impl";
  pub(super) const EXTERN: &str = "extern";
  pub(super) const WHILE: &str = "while";
  pub(super) const FOR: &str = "for";
  pub(super) const IF: &str = "if";
  pub(super) const ELSE: &str = "else";
  pub(super) const DO: &str = "do";
  pub(super) const LOOP: &str = "loop";
  pub(super) const UNTIL: &str = "until";
  pub(super) const BREAK: &str = "break";
  pub(super) const CONTINUE: &str = "continue";
  pub(super) const RETURN: &str = "return";
  pub(super) const TEMPLATE: &str = "template";
  pub(super) const EXTENDS: &str = "extends";
  pub(super) const CONST: &str = "const";
  pub(super) const MUT: &str = "mut";
}

#[derive(Debug)]
pub(crate) enum Keyword {
  Type,
  Import,
  Export,
  From,
  Struct,
  Class,
  Private,
  Protected,
  Public,
  Abstract,
  Interface,
  Namespace,
  Implements,
  Impl,
  Extern,
  While,
  For,
  If,
  Else,
  Do,
  Loop,
  Until,
  Break,
  Continue,
  Return,
  Template,
  Extends,
  Const,
  Mut,
}

impl std::string::ToString for Keyword {
  fn to_string(&self) -> String {
    match self {
      Self::Type => keywords::TYPE,
      Self::Import => keywords::IMPORT,
      Self::Export => keywords::EXPORT,
      Self::From => keywords::FROM,
      Self::Struct => keywords::STRUCT,
      Self::Class => keywords::CLASS,
      Self::Private => keywords::PRIVATE,
      Self::Protected => keywords::PROTECTED,
      Self::Public => keywords::PUBLIC,
      Self::Abstract => keywords::ABSTRACT,
      Self::Interface => keywords::INTERFACE,
      Self::Namespace => keywords::NAMESPACE,
      Self::Implements => keywords::IMPLEMENTS,
      Self::Impl => keywords::IMPL,
      Self::Extern => keywords::EXTERN,
      Self::While => keywords::WHILE,
      Self::For => keywords::FOR,
      Self::If => keywords::IF,
      Self::Else => keywords::ELSE,
      Self::Do => keywords::DO,
      Self::Loop => keywords::LOOP,
      Self::Until => keywords::UNTIL,
      Self::Break => keywords::BREAK,
      Self::Continue => keywords::CONTINUE,
      Self::Return => keywords::RETURN,
      Self::Template => keywords::TEMPLATE,
      Self::Extends => keywords::EXTENDS,
      Self::Const => keywords::CONST,
      Self::Mut => keywords::MUT,
    }.to_string()
  }
}

impl TryFrom<&String> for Keyword {
  type Error = ();

  fn try_from(value: &String) -> Result<Self, Self::Error> {
    match value.as_str() {
      keywords::TYPE => Ok(Self::Type),
      keywords::IMPORT => Ok(Self::Import),
      keywords::EXPORT => Ok(Self::Export),
      keywords::FROM => Ok(Self::From),
      keywords::STRUCT => Ok(Self::Struct),
      keywords::CLASS => Ok(Self::Class),
      keywords::PRIVATE => Ok(Self::Private),
      keywords::PROTECTED => Ok(Self::Protected),
      keywords::PUBLIC => Ok(Self::Public),
      keywords::ABSTRACT => Ok(Self::Abstract),
      keywords::INTERFACE => Ok(Self::Interface),
      keywords::NAMESPACE => Ok(Self::Namespace),
      keywords::IMPLEMENTS => Ok(Self::Implements),
      keywords::IMPL => Ok(Self::Impl),
      keywords::EXTERN => Ok(Self::Extern),
      keywords::WHILE => Ok(Self::While),
      keywords::FOR => Ok(Self::For),
      keywords::IF => Ok(Self::If),
      keywords::ELSE => Ok(Self::Else),
      keywords::DO => Ok(Self::Do),
      keywords::LOOP => Ok(Self::Loop),
      keywords::UNTIL => Ok(Self::Until),
      keywords::BREAK => Ok(Self::Break),
      keywords::CONTINUE => Ok(Self::Continue),
      keywords::RETURN => Ok(Self::Return),
      keywords::TEMPLATE => Ok(Self::Template),
      keywords::EXTENDS => Ok(Self::Extends),
      keywords::CONST => Ok(Self::Const),
      keywords::MUT => Ok(Self::Mut),
      _ => Err(())
    }
  }
}

#[derive(Debug, TypeName, Clone)]
pub(crate) enum Literal {
  Integer(u64),
  FloatingPoint(f64),
  UnicodeString(String),
  CString(String),
  ByteString(String),
  UnicodeChar(char),
  ByteChar(u8),
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum TokenEnum {
  Comment { ty: CommentType, content: String },
  Literal(Literal),
  Whitespace(String),
  Keyword(Keyword),
  Identifier(String),
  Operator(Operator),
  Punctuation(Punctuation),
  Grouping(Grouping),
  Invalid(String),
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Token {
  pub(crate) token: TokenEnum,
  pub(crate) span: Span,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum GroupingType {
  Parenthesis,
  Bracket,
  CurlyBrace
}

#[derive(Debug)]
pub(crate) enum Grouping {
  Open(GroupingType),
  Close(GroupingType)
}

pub(crate) mod groupings {
  pub(crate) const OPEN_PARENTHESIS: char = '(';
  pub(crate) const CLOSE_PARENTHESIS: char = ')';
  pub(crate) const OPEN_BRACKET: char = '[';
  pub(crate) const CLOSE_BRACKET: char = ']';
  pub(crate) const OPEN_CURLY_BRACE: char = '{';
  pub(crate) const CLOSE_CURLY_BRACE: char = '}';
}

impl std::string::ToString for Grouping {
  fn to_string(&self) -> String {
    match self {
      Self::Open(GroupingType::Parenthesis) => groupings::OPEN_PARENTHESIS,
      Self::Close(GroupingType::Parenthesis) => groupings::CLOSE_PARENTHESIS,
      Self::Open(GroupingType::Bracket) => groupings::OPEN_BRACKET,
      Self::Close(GroupingType::Bracket) => groupings::CLOSE_BRACKET,
      Self::Open(GroupingType::CurlyBrace) => groupings::OPEN_CURLY_BRACE,
      Self::Close(GroupingType::CurlyBrace) => groupings::CLOSE_CURLY_BRACE,
    }.to_string()
  }
}

impl TryFrom<char> for Grouping {
  type Error = ();

  fn try_from(value: char) -> Result<Self, Self::Error> {
    match value {
      groupings::OPEN_PARENTHESIS => Ok(Self::Open(GroupingType::Parenthesis)),
      groupings::CLOSE_PARENTHESIS => Ok(Self::Close(GroupingType::Parenthesis)),
      groupings::OPEN_BRACKET => Ok(Self::Open(GroupingType::Bracket)),
      groupings::CLOSE_BRACKET => Ok(Self::Close(GroupingType::Bracket)),
      groupings::OPEN_CURLY_BRACE => Ok(Self::Open(GroupingType::CurlyBrace)),
      groupings::CLOSE_CURLY_BRACE => Ok(Self::Close(GroupingType::CurlyBrace)),
      _ => Err(())
    }
  }
}

impl Token {
  pub fn variant(&self) -> &TokenEnum {
    &self.token
  }
}
