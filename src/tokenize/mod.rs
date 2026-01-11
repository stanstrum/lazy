mod token;

use std::collections::VecDeque;
use std::io::Read;

use crate::lang::ModuleId;
use crate::bufreader::{BufferedUtf8MetadataReader};
use crate::string_pool::StringPool;
use crate::tokenize::token::{GroupingKind, GroupingType};
use token::{Keyword};

pub use token::{Token, TokenSpan, Position};

#[derive(Debug)]
pub struct Tokenizer<'pool, const N: usize, T: Read> {
  pool: &'pool StringPool,
  id: ModuleId,
  meta_reader: BufferedUtf8MetadataReader<N, T>,
  state: State,
  next: Option<char>,
  toks: VecDeque<TokenSpan>,
}

#[derive(Debug)]
pub enum Error {
  IO,
}

#[derive(Debug)]
enum State {
  Base,
  Whitespace {
    start: Position,
  },
  Text {
    content: String,
    start: Position,
  },
  Operator {
    content: String,
    start: Position,
  },
  LineComment {
    content: String,
    start: Position,
  },
}

impl<'pool, const N: usize, T: Read> Tokenizer<'pool, N, T> {
  pub fn new(pool: &'pool StringPool, id: ModuleId, meta_reader: BufferedUtf8MetadataReader<N, T>) -> Self {
    Self {
      pool,
      id,
      meta_reader,
      state: State::Base,
      next: None,
      toks: VecDeque::new(),
    }
  }

  fn take(&mut self) -> Result<Option<char>, Error> {
    if let Some(next) = self.next.take() {
      return Ok(Some(next));
    };

    match self.meta_reader.next() {
      Some(Ok(ch)) => Ok(Some(ch)),
      Some(Err(_)) => Err(Error::IO),
      None => Ok(None),
    }
  }

  fn save(&mut self, ch: char) {
    assert!(self.next.is_none());

    self.next = Some(ch);
  }

  fn retry(&mut self, ch: char, state: State) {
    self.save(ch);
    self.state = state;
  }

  fn push_here(&mut self, tok: Token, start: Position) {
    self.toks.push_back(TokenSpan {
      tok,
      start,
      end: self.pos(),
      module: self.id,
    });
  }

  fn pos(&self) -> Position {
    Position::new(&self.meta_reader.meta)
  }
}

impl<'a, const N: usize, T: Read> Iterator for Tokenizer<'a, N, T> {
  type Item = Result<TokenSpan, Error>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if let Some(tok) = self.toks.pop_front() {
        return Some(Ok(tok));
      };

      let ch = match self.take() {
        Ok(Some(ch)) => ch,
        Ok(None) => return None,
        Err(_) => return Some(Err(Error::IO)),
      };

      match (&mut self.state, ch) {
        // Base
        // -> Text
        (State::Base, 'a'..='z' | 'A'..='Z' | '_') => {
          self.retry(ch, State::Text {
            start: self.pos(),
            content: String::new(),
          });
        },
        // -> Whitespace
        (State::Base, ' ' | '\t') => {
          self.retry(ch, State::Whitespace {
            start: self.pos(),
          });
        },
        // -> Operator
        (State::Base, '-' | '/') => {
          self.retry(ch, State::Operator {
            start: self.pos(),
            content: String::new(),
          });
        },
        // -> Grouping
        (State::Base, '(' | '[' | '{' | '}' | ']' | ')') => {
          let tok = match ch {
            '(' => Token::Grouping(GroupingType::Open(GroupingKind::Parenthesis)),
            '[' => Token::Grouping(GroupingType::Open(GroupingKind::Bracket)),
            '{' => Token::Grouping(GroupingType::Open(GroupingKind::Brace)),
            '}' => Token::Grouping(GroupingType::Close(GroupingKind::Brace)),
            ']' => Token::Grouping(GroupingType::Close(GroupingKind::Bracket)),
            ')' => Token::Grouping(GroupingType::Close(GroupingKind::Parenthesis)),
            _ => unreachable!(),
          };

          self.push_here(tok, self.pos());
        },
        // -> Newline
        (State::Base, '\n') => {
          let indentation = self.meta_reader.meta.whitespace as isize;

          loop {
            if let Some(result) = self.meta_reader.next() {
              let Ok(ch) = result else {
                return Some(Err(Error::IO));
              };

              if !matches!(ch, ' ' | '\t') {
                let difference = self.meta_reader.meta.whitespace as isize - indentation;

                self.push_here(Token::Indent(difference), self.pos());
                self.retry(ch, State::Base);
                break;
              };
            };
          };
        },
        // Text
        (State::Text { content, .. },
          'a'..='z' | 'A'..='Z' | '0'..='9' | '_') => {
          content.push(ch);
        },
        // -> Base
        (State::Text { content, start }, _) => {
          let tok = if let Some(keyword) = Keyword::from_str(&content) {
            Token::Keyword(keyword)
          } else {
            let id = self.pool.insert(content);
            Token::Identifier(id)
          };

          let start = *start;
          self.push_here(tok, start);
          self.retry(ch, State::Base);
        },
        // Whitespace
        (State::Whitespace { .. }, ' ' | '\t') => {
          // do nothing
        },
        // -> Base
        (State::Whitespace { start }, _) => {
          let start = *start;
          self.push_here(Token::Whitespace, start);
          self.retry(ch, State::Base);
        },
        // Operator
        (State::Operator { content, start }, _) => {
          let start = *start;
          match (content.as_str(), ch) {
            | ("", '-')
            | ("-", '>')
            | ("", '/')
            | ("/", '/')
            | ("/", '*')
              => content.push(ch),
            ("->", _) => {
              self.push_here(Token::Operator(token::Operator::RightArrow), start);
              self.retry(ch, State::Base);
            },
            ("//", _) => {
              self.retry(ch, State::LineComment {
                content: String::new(),
                start,
              });
            },
            _ => todo!("operator {content:?} and {ch:?}"),
          }
        },
        // Line comment
        (State::LineComment { .. }, '\n') => {
          let State::LineComment { mut content, start } =
            std::mem::replace(&mut self.state, State::Base) else {
              unreachable!()
          };

          // trim in place
          let trim_chars = content.len() - content.trim_start().len();
          content.replace_range(0..trim_chars, "");
          content.truncate(content.trim_end().len());

          self.push_here(Token::Comment(content), start);
          self.save(ch);
        },
        (State::LineComment { content, .. }, _) => {
          content.push(ch);
        },
        // Fallthrough
        other => todo!("tokenize state {other:?}"),
      }
    }
  }
}
