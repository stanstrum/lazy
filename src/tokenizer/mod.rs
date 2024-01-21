/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fs::File;

use snafu::prelude::*;
use utf8_read::Reader;

#[derive(Snafu, Debug)]
pub(crate) enum TokenizationError {
  IOError { error: utf8_read::Error }
}

#[derive(Debug)]
pub(crate) struct Span {
  start: usize,
  end: usize
}

#[derive(Debug)]
pub(crate) enum Operator {
  RightArrow,
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

#[derive(Debug)]
pub(crate) enum TokenEnum {
  Comment { ty: CommentType, content: String },
  Whitespace(String),
  Keyword,
  Identifier(String),
  Operator(Operator),
  Punctuation(Punctuation),
  Grouping(Grouping),
}

#[derive(Debug)]
pub(crate) struct Token {
  token: TokenEnum,
  span: Span
}

impl From<utf8_read::Error> for TokenizationError {
  fn from(error: utf8_read::Error) -> Self {
    Self::IOError { error }
  }
}

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

#[derive(Debug)]
enum State {
  Base,
  CommentBegin {
    start: usize,
  },
  MultilineComment {
    start: usize,
    content: String,
  },
  LineComment {
    start: usize,
    content: String,
  },
  MultilineCommentEnding {
    start: usize,
    content: String
  },
  Text {
    start: usize,
    content: String
  },
  Operator {
    start: usize,
    content: String,
  },
  // LineComment(String),
  Whitespace {
    start: usize,
    content: String
  },
}

pub(crate) fn tokenize(reader: &mut Reader<File>) -> Result<Vec<Token>, TokenizationError> {
  let mut toks: Vec<Token> = vec![];

  let mut state = State::Base;

  for (i, ch) in reader.into_iter().enumerate() {
    let ch = ch?;

    let mut add_tok = |start: &usize, token: TokenEnum| {
      toks.push(Token {
        token,
        span: Span {
          start: *start,
          end: i,
        }
      });
    };

    print!("{}", ch);

    loop {
      match (&mut state, ch) {
        (State::Base, '/') => {
          state = State::CommentBegin {
            start: i
          };
        },
        (State::CommentBegin { start }, '*') => {
          state = State::MultilineComment {
            start: *start,
            content: String::new()
          };
        },
        (State::MultilineComment { start, content }, '*') => {
          state = State::MultilineCommentEnding {
            start: *start,
            content: content.to_owned()
          };
        },
        (State::MultilineComment { content, .. }, _) => {
          content.push(ch);
        },
        (State::MultilineCommentEnding { start, content }, '/') => {
          let tok = TokenEnum::Comment {
            ty: CommentType::Multiline,
            content: content.to_owned()
          };

          add_tok(start, tok);

          state = State::Base;
        },
        (State::MultilineCommentEnding { start, content }, _) => {
          state = State::MultilineComment {
            start: *start,
            content: format!("{content}*{ch}")
          };
        },
        (State::CommentBegin { start }, '/') => {
          state = State::LineComment {
            start: *start,
            content: String::new()
          };
        },
        (State::LineComment { start, content }, '\n') => {
          let tok = TokenEnum::Comment {
            ty: CommentType::Line,
            content: content.to_owned()
          };

          add_tok(start, tok);

          state = State::Base;
        },
        (State::LineComment { content, .. }, _) => {
          content.push(ch);
        },
        (State::Base, ' ' | '\n' | '\t') => {
          state = State::Whitespace {
            start: i,
            content: String::from(ch)
          };
        },
        (State::Whitespace { content, .. }, ' ' | '\n' | '\t') => {
          content.push(ch);
        },
        (State::Whitespace { start, content }, _) => {
          let tok = TokenEnum::Whitespace(content.to_owned());

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Base, 'a'..='z' | 'A'..='Z' | '_') => {
          state = State::Text {
            start: i,
            content: String::from(ch)
          };
        },
        (State::Text { content, .. }, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') => {
          content.push(ch);
        },
        (State::Text { start, content }, _) => {
          let tok = TokenEnum::Identifier(content.to_owned());

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Base, '!' | '%' | '^' | '&' | '*' | '-' | '+' | '=' | '<' | '>' | '|' | '/' | ':' | '.' | '?') => {
          state = State::Operator {
            start: i,
            content: String::from(ch)
          };
        },
        (State::Operator { start, content }, '>') if content == "-" => {
          let tok = TokenEnum::Operator(Operator::RightArrow);

          add_tok(start, tok);

          state = State::Base;
        },
        (State::Operator { content, .. }, ':') if content == ":" => {
          todo!("double colon")
        },
        (State::Operator { start, content }, _) if content == ":" => {
          let tok = TokenEnum::Punctuation(Punctuation::Colon);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Base, ',') => {
          let tok = TokenEnum::Punctuation(Punctuation::Comma);

          add_tok(&i, tok);
        },
        (State::Base, '{') => {
          let tok = TokenEnum::Grouping(Grouping::Open(GroupingType::CurlyBrace));

          add_tok(&i, tok);
        },
        (State::Base, '}') => {
          let tok = TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace));

          add_tok(&i, tok);
        },
        (State::Base, ';') => {
          let tok = TokenEnum::Punctuation(Punctuation::Comma);

          add_tok(&i, tok);
        },
        _ => {
          dbg!(&toks);
          todo!("{state:#?}, {ch:#?}")
        }
      };

      break;
    };
  };

  Ok(toks)
}
