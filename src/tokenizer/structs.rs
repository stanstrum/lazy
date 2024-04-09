use typename::TypeName;

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Span {
  pub start: usize,
  pub end: usize
}

#[derive(Debug)]
pub(crate) enum Operator {
  BindAssign,
  RightArrow,
  SingleAnd,
  Increment,
  Add,
  Decrement,
  Subtract,
  Exponent,
  Multiply,
  // integer decrement?
  Divide,
  Equality,
  Equals,
  Range,
  Dot,
  Separator,
}

#[derive(Debug)]
pub(crate) enum Punctuation {
  Colon,
  Comma,
  Semicolon
}

#[derive(Debug)]
pub(crate) enum CommentType {
  Line,
  Multiline
}

mod keywords {
  pub(super) const TYPE: &str = "type";
  pub(super) const IMPORT: &str = "import";
  pub(super) const EXPORT: &str = "export";
  pub(super) const FROM: &str = "from";
  pub(super) const STRUCT: &str = "struct";
  pub(super) const CLASS: &str = "class";
  pub(super) const NAMESPACE: &str = "namespace";
  pub(super) const IMPL: &str = "impl";
}

#[derive(Debug)]
pub(crate) enum Keyword {
  Type,
  Import,
  Export,
  From,
  Struct,
  Class,
  Namespace,
  Impl,
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
      Self::Namespace => keywords::NAMESPACE,
      Self::Impl => keywords::IMPL,
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
      keywords::NAMESPACE => Ok(Self::Namespace),
      keywords::IMPL => Ok(Self::Impl),
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
  ByteString(String)
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
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Token {
  pub token: TokenEnum,
  pub span: Span
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
  pub fn variant<'a>(&'a self) -> &'a TokenEnum {
    &self.token
  }
}
