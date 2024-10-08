mod span;
mod consts;

pub(crate) use consts::*;
pub(crate) use span::*;

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum TokenKind {
  Whitespace,
  Identifier(String),
  Operator(Operator),
  Keyword(Keyword),
  Comment(String),
  Punctuation(Punctuation),
  Grouping(Grouping),
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Token {
  pub kind: TokenKind,
  pub span: Span,
}
