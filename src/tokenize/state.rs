use std::io::Read;

use crate::tokenize::token::{
  Token,
  TokenSpan,
  Position,
  Keyword,
  Operator,
  GroupingKind,
  GroupingType,
  NumericKind,
};

use super::{Tokenizer, Error};

fn trim_in_place(string: &mut String) {
  // trim end
  string.truncate(string.trim_end().len());

  // trim beginning
  let trim_chars = string.len() - string.trim_start().len();
  string.replace_range(0..trim_chars, "");
}

#[derive(Debug)]
pub(super) enum State {
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
  MultilineComment {
    content: String,
    start: Position,
  },
  Numeric {
    kind: Option<NumericKind>,
    content: String,
    start: Position,
  },
}

impl<'pool, const N: usize, T: Read> Tokenizer<'pool, N, T> {
  pub(super) fn do_state(&mut self) -> Option<Result<TokenSpan, Error>> {
    loop {
      if let Some(tok) = self.toks.pop_front() {
        return Some(Ok(tok));
      };

      let ch = match self.take() {
        Ok(Some(ch)) => ch,
        Ok(None) => return self.end_indent(),
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
            let result = self.meta_reader.next()?;

            let Ok(ch) = result else {
              return Some(Err(Error::IO));
            };

            if !matches!(ch, ' ' | '\t') {
              let difference = self.meta_reader.meta.whitespace as isize - indentation;

              self.indentation += difference;

              self.push_here(Token::Indent(difference), self.pos());
              self.retry(ch, State::Base);
              break;
            };
          };
        },
        // -> Numeric
        (State::Base, '0'..='9') => {
          self.retry(ch, State::Numeric {
            kind: None,
            content: String::new(),
            start: self.pos(),
          });
        },
        // Text
        (State::Text { content, .. },
          'a'..='z' | 'A'..='Z' | '0'..='9' | '_') => {
          content.push(ch);
        },
        // -> Base
        (State::Text { content, start }, _) => {
          let tok = if let Some(keyword) = Keyword::from_str(content) {
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
              self.push_here(Token::Operator(Operator::RightArrow), start);
              self.retry(ch, State::Base);
            },
            ("//", _) => {
              self.retry(ch, State::LineComment {
                content: String::new(),
                start,
              });
            },
            ("/*", _) => {
              self.retry(ch, State::MultilineComment {
                content: String::new(),
                start,
              });
            }
            _ => todo!("operator {content:?} and {ch:?}"),
          }
        },
        // Line comment
        (State::LineComment { .. }, '\n') => {
          let State::LineComment { mut content, start } =
            std::mem::replace(&mut self.state, State::Base) else {
              unreachable!()
          };

          trim_in_place(&mut content);

          let comment_id = self.pool.insert_comment(content);
          self.push_here(Token::Comment(comment_id), start);
          self.save(ch);
        },
        (State::LineComment { content, .. }, _) => {
          content.push(ch);
        },
        // Multiline comment
        (State::MultilineComment { content, .. }, _) if content.ends_with("*/") => {
          let State::MultilineComment { mut content, start } =
            std::mem::replace(&mut self.state, State::Base) else {
              unreachable!()
          };

          // this could just be an unwrap given the if-guard
          if let Some(slice) = content.strip_suffix("*/") {
            content.truncate(slice.len());
          };
          trim_in_place(&mut content);

          let comment_id = self.pool.insert_comment(content);

          self.push_here(Token::Comment(comment_id), start);
          self.save(ch);
        },
        (State::MultilineComment { content, .. }, _) => {
          content.push(ch);
        },
        // Numeric
        (
          State::Numeric { content, kind, ..},
          'b' | 't' | 's' | 'o' | 'x' | 'r'
        ) if kind.is_none() && content == "0" => {
          content.clear();
          *kind = Some(match ch {
            'b' => NumericKind::Binary,
            't' => NumericKind::Ternary,
            's' => NumericKind::Seximal,
            'o' => NumericKind::Octal,
            'x' => NumericKind::Hexadecimal,
            'r' => NumericKind::Roman,
            _ => unreachable!("numeric code: {ch}"),
          });
        },
        | (State::Numeric { content, kind: Some(NumericKind::Hexadecimal), .. }, 'a'..='f' | 'A'..='F')
        | (State::Numeric { content, .. }, '0'..='9' ) => {
          content.push(ch);
        },
        (State::Numeric { content, .. }, '.') if !content.contains('.') => {
          content.push(ch);
        },
        (State::Numeric { content, .. }, '.') if content.ends_with('.') => {
          content.pop();

          let State::Numeric {
            kind,
            content,
            start,
          } = std::mem::replace(&mut self.state, State::Base) else {
            unreachable!()
          };

          let tok = match self.parse_and_push(kind, &content) {
            Ok(tok) => tok,
            Err(error) => return Some(Err(error)),
          };

          self.push_here(tok, start);
          self.push_here(Token::Operator(Operator::Range), self.pos());
        },
        // -> Base
        (State::Numeric { .. }, _) => {
          let State::Numeric {
            kind,
            content,
            start,
          } = std::mem::replace(&mut self.state, State::Base) else {
            unreachable!()
          };

          let tok = match self.parse_and_push(kind, &content) {
            Ok(tok) => tok,
            Err(error) => return Some(Err(error)),
          };

          self.push_here(tok, start);
          self.save(ch);
        },
        // Fallthrough
        other => todo!("tokenize state {other:?}"),
      }
    }
  }
}
