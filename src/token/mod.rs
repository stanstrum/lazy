mod keyword;
mod preprocessor;

use snafu::Whatever;

use super::*;

#[derive(Debug)]
pub(super) enum GroupingKind {
  Curly,
  Bracket,
  Parenthesis,
}

#[derive(Debug)]
pub(super) enum Grouping {
  Open(GroupingKind),
  Close(GroupingKind),
}

#[derive(Debug)]
pub(super) enum Punctuation {
  Semicolon,
  Colon,
  Comma,
  RightArrow,
  Bollocks,
}

#[derive(Debug)]
pub(super) enum TokenKind {
  Whitespace(String),
  Keyword(keyword::Keyword),
  Identifier(String),
  Grouping(Grouping),
  Punctuation(Punctuation),
}

pub(super) type Token = (TokenKind, Span);

#[derive(Debug, Clone, Copy)]
pub(super) struct SourceMark {
  pub line: usize,
  pub column: usize,
  pub start_of_line_byte: usize,
  pub indentation_level: usize,
}

#[derive(Debug)]
pub(super) struct Span {
  pub id: usize,
  pub start: SourceMark,
  pub end: SourceMark,
}

#[derive(Debug)]
pub(super) struct Tokenizer<T: Iterator<Item = (char, usize)>> {
  id: usize,
  mark: SourceMark,
  indent_buf: String,
  iter: T,
  tokens: VecDeque<Token>,
  start_of_line: bool,
  tail_call: Option<char>,
}

impl<T: Iterator<Item = (char, usize)>> Tokenizer<T> {
  pub(super) fn new(id: usize, iter: T) -> Self {
    Self {
      id,
      mark: SourceMark {
        line: 0,
        column: 0,
        start_of_line_byte: 0,
        indentation_level: 0,
      },
      indent_buf: String::new(),
      iter,
      tokens: VecDeque::new(),
      start_of_line: true,
      tail_call: None,
    }
  }

  fn base(&mut self) {
    let start = self.mark;

    let Some(ch) = self.read_ch() else {
      return;
    };

    match ch {
      'a'..='z' | 'A'..='Z' | '_' => self.word(ch),
      ' ' | '\t' | '\n' => self.whitespace(ch),
      '(' => self.push_token(
        TokenKind::Grouping(Grouping::Open(GroupingKind::Parenthesis)),
        start,
      ),
      ')' => self.push_token(
        TokenKind::Grouping(Grouping::Close(GroupingKind::Parenthesis)),
        start,
      ),
      '[' => self.push_token(
        TokenKind::Grouping(Grouping::Open(GroupingKind::Bracket)),
        start,
      ),
      ']' => self.push_token(
        TokenKind::Grouping(Grouping::Close(GroupingKind::Bracket)),
        start,
      ),
      '{' => self.push_token(
        TokenKind::Grouping(Grouping::Open(GroupingKind::Curly)),
        start,
      ),
      '}' => self.push_token(
        TokenKind::Grouping(Grouping::Close(GroupingKind::Curly)),
        start,
      ),
      ';' => self.push_token(TokenKind::Punctuation(Punctuation::Semicolon), start),
      _ => todo!("{ch:?}"),
    };
  }

  fn word(&mut self, ch: char) {
    let mut word = String::from(ch);
    let start = self.mark;

    loop {
      match self.read_ch() {
        Some(ch @ ('a'..='z' | 'A'..='Z' | '_' | '0'..='9')) => {
          word.push(ch);
        },
        other => break self.tail_call = other,
      };
    }

    let token = if let Ok(keyword) = keyword::Keyword::try_from(word.as_str()) {
      TokenKind::Keyword(keyword)
    } else {
      TokenKind::Identifier(word)
    };

    self.push_token(token, start);
  }

  fn whitespace(&mut self, ch: char) {
    let mut space = String::from(ch);
    let start = self.mark;

    loop {
      match self.read_ch() {
        Some(ch @ (' ' | '\t' | '\n')) => space.push(ch),
        other => break self.tail_call = other,
      };
    }

    self.push_token(TokenKind::Whitespace(space), start);
  }
}
