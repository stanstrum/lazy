mod structs;
mod to_string;

pub(crate) mod error;

use std::fs::File;
use utf8_read::Reader;

mod state;
use state::*;

pub(crate) use structs::*;
pub(crate) use error::TokenizationError;

pub(crate) fn tokenize(reader: &mut Reader<File>) -> Result<Vec<Token>, TokenizationError> {
  let mut state = State::Base;
  let mut toks: Vec<Token> = vec![];

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
        (State::Text { start, content}, _) if content == "type" => {
          add_tok(start, TokenEnum::Keyword(Keyword::Type));

          state = State::Base;
          continue;
        },
        (State::Text { start, content }, _) => {
          let tok = TokenEnum::Identifier(content.to_owned());

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Base, '!' | '%' | '^' | '&' | '*' | '-' | '+' | '=' | '<' | '>' | '|' | ':' | '.' | '?') => {
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
        (State::Operator { start, content, .. }, '=') if content == ":" => {
          add_tok(start, TokenEnum::Operator(Operator::BindAssign));

          state = State::Base;
        },
        (State::Operator { start, content }, _) if content == ":" => {
          let tok = TokenEnum::Punctuation(Punctuation::Colon);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Operator { start, content }, _) if content == "&" => {
          let tok = TokenEnum::Operator(Operator::SingleAnd);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Base, ',') => {
          let tok = TokenEnum::Punctuation(Punctuation::Comma);

          add_tok(&i, tok);
        },
        (State::Base,
          | groupings::OPEN_PARENTHESIS
          | groupings::CLOSE_PARENTHESIS
          | groupings::OPEN_BRACKET
          | groupings::CLOSE_BRACKET
          | groupings::OPEN_CURLY_BRACE
          | groupings::CLOSE_CURLY_BRACE
        ) => {
          let tok = TokenEnum::Grouping(Grouping::try_from(ch).unwrap());

          add_tok(&i, tok);
        }
        (State::Base, ';') => {
          let tok = TokenEnum::Punctuation(Punctuation::Semicolon);

          add_tok(&i, tok);
        },
        (State::Base, '0'..='9') => {
          state = State::NumericLiteral {
            start: i,
            ty: NumericType::Decimal,
            content: String::from(ch)
          };
        },
        (State::NumericLiteral { .. }, '_') => {
          // ignore value separator in numerics
        },
        (
          State::NumericLiteral {
            content,
            ty: NumericType::Decimal,
            ..
          },
          '0'..='9'
        ) => {
          content.push(ch);
        },
        (State::NumericLiteral { start, ty: NumericType::FloatingPoint, content }, _) => {
          let value: f64 = content.parse()
            .expect("floating-point literal parsing failed");

          let tok = TokenEnum::Literal(Literal::FloatingPoint(value));

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::NumericLiteral { start, ty, content }, _) => {
          let radix = match ty {
            NumericType::Binary => 2,
            NumericType::Octal => 8,
            NumericType::Decimal => 10,
            NumericType::Hexadecimal => 16,
            NumericType::FloatingPoint => todo!(),
          };

          let value = u64::from_str_radix(content, radix)
            .expect("integer literal parsing failed");

          let tok = TokenEnum::Literal(Literal::Integer(value));

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        _ => {
          println!();

          dbg!(&toks);
          println!("state: {state:#?}, ch: {ch:#?}");

          panic!("unknown tokenization state");
        }
      };

      break;
    };
  };

  Ok(toks)
}
