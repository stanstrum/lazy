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
  SingleAnd
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
}

#[derive(Debug)]
pub(crate) enum Keyword {
  Type,
  Import,
  Export,
  From,
}

impl std::string::ToString for Keyword {
  fn to_string(&self) -> String {
    match self {
      Keyword::Type => keywords::TYPE,
      Keyword::Import => keywords::IMPORT,
      Keyword::Export => keywords::EXPORT,
      Keyword::From => keywords::FROM,
    }.to_string()
  }
}

impl TryFrom<String> for Keyword {
  type Error = ();

  fn try_from(value: String) -> Result<Self, Self::Error> {
    match value.as_str() {
      keywords::TYPE => Ok(Keyword::Type),
      keywords::IMPORT => Ok(Keyword::Import),
      keywords::EXPORT => Ok(Keyword::Export),
      keywords::FROM => Ok(Keyword::From),
      _ => Err(())
    }
  }
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum TokenEnum {
  Comment { ty: CommentType, content: String },
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
      Grouping::Open(GroupingType::Parenthesis) => groupings::OPEN_PARENTHESIS,
      Grouping::Close(GroupingType::Parenthesis) => groupings::CLOSE_PARENTHESIS,
      Grouping::Open(GroupingType::Bracket) => groupings::OPEN_BRACKET,
      Grouping::Close(GroupingType::Bracket) => groupings::CLOSE_BRACKET,
      Grouping::Open(GroupingType::CurlyBrace) => groupings::OPEN_CURLY_BRACE,
      Grouping::Close(GroupingType::CurlyBrace) => groupings::CLOSE_CURLY_BRACE,
    }.to_string()
  }
}

impl TryFrom<char> for Grouping {
  type Error = ();

  fn try_from(value: char) -> Result<Self, Self::Error> {
    match value {
      groupings::OPEN_PARENTHESIS => Ok(Grouping::Open(GroupingType::Parenthesis)),
      groupings::CLOSE_PARENTHESIS => Ok(Grouping::Close(GroupingType::Parenthesis)),
      groupings::OPEN_BRACKET => Ok(Grouping::Open(GroupingType::Bracket)),
      groupings::CLOSE_BRACKET => Ok(Grouping::Close(GroupingType::Bracket)),
      groupings::OPEN_CURLY_BRACE => Ok(Grouping::Open(GroupingType::CurlyBrace)),
      groupings::CLOSE_CURLY_BRACE => Ok(Grouping::Close(GroupingType::CurlyBrace)),
      _ => Err(())
    }
  }
}

impl Token {
  pub fn variant<'a>(&'a self) -> &'a TokenEnum {
    &self.token
  }
}
