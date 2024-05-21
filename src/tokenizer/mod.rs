mod structs;
mod to_string;

pub(crate) mod error;

use std::fs::File;
use utf8_read::Reader;

mod state;
use state::*;

pub(crate) use structs::*;
pub(crate) use error::TokenizationError;

use crate::colors::Color;

use self::error::InvalidSourceSnafu;

pub(crate) fn stringify(tokens: &[Token]) -> String {
  let mut source = String::new();

  for token in tokens.iter() {
    source += token.to_string().as_str();
  };

  // Fixes some odd issues later on by preventing the error reporter
  // from breaking if the source doesn't end on a newline.  :shrug:
  source.push('\n');

  source
}

pub(crate) fn create_color_stream(tokens: &[Token]) -> Vec<(usize, Color)> {
  let mut color_stream = vec![];

  for token in tokens {
    let color = {
      match &token.token {
        TokenEnum::Comment { .. } => Color::DarkGrey,
        TokenEnum::Literal(_) => Color::Mint,
        TokenEnum::Keyword(_) => Color::LightRed,
        TokenEnum::Identifier(_) => Color::Creme,
        TokenEnum::Operator(Operator::Separator) => Color::Creme,
        TokenEnum::Operator(_) => Color::LightBlue,
        _ => Color::Clear
      }
    };

    color_stream.push((token.span.start, color));
  };

  color_stream
}

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
        (State::Base, ' ' | '\n' | '\r' | '\t') => {
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
        (State::Text { start, content }, '"') if content == "b" => {
          state = State::StringLiteral {
            start: *start,
            ty: StringType::Bytes,
            content: String::new()
          };
        },
        (State::Text { start, content }, '"') if content == "c" => {
          state = State::StringLiteral {
            start: *start,
            ty: StringType::C,
            content: String::new()
          };
        },
        (State::Text { start, content }, '\'') if content == "b" => {
          state = State::CharLiteral {
            start: *start,
            ty: CharType::Byte,
            content: String::new()
          };
        },
        (State::Text { start, ref content }, _) => {
          let tok = {
            if let Ok(keyword) = Keyword::try_from(content) {
              TokenEnum::Keyword(keyword)
            } else {
              TokenEnum::Identifier(content.to_owned())
            }
          };

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Base, '!' | '%' | '^' | '&' | '*' | '-' | '+' | '=' | '<' | '>' | /* '|' | */ ':' | '.' | '?') => {
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
        (State::Operator { content, start }, ':') if content == ":" => {
          let tok = TokenEnum::Operator(Operator::Separator);

          add_tok(start, tok);

          state = State::Base;
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
        (State::Operator { start, content }, '+') if content == "+" => {
          let tok = TokenEnum::Operator(Operator::Increment);

          add_tok(start, tok);

          state = State::Base;
        },
        (State::Operator { start, content }, _) if content == "+" => {
          let tok = TokenEnum::Operator(Operator::Add);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Operator { start, content }, '-') if content == "-" => {
          let tok = TokenEnum::Operator(Operator::Decrement);

          add_tok(start, tok);

          state = State::Base;
        },
        (State::Operator { start, content }, _) if content == "-" => {
          let tok = TokenEnum::Operator(Operator::Subtract);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Operator { start, content }, '*') if content == "*" => {
          let tok = TokenEnum::Operator(Operator::Exponent);

          add_tok(start, tok);

          state = State::Base;
        },
        (State::Operator { start, content }, _) if content == "*" => {
          let tok = TokenEnum::Operator(Operator::Multiply);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::CommentBegin { start }, _) => {
          let tok = TokenEnum::Operator(Operator::Divide);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Operator { start, content }, '=') if content == "=" => {
          let tok = TokenEnum::Operator(Operator::Equality);

          add_tok(start, tok);

          state = State::Base;
        },
        (State::Operator { start, content }, _) if content == "=" => {
          let tok = TokenEnum::Operator(Operator::Equals);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Operator { start, content }, '.') if content == ".." => {
          let tok = TokenEnum::Punctuation(Punctuation::VariadicEllipsis);

          add_tok(start, tok);

          state = State::Base;
        },
        (State::Operator { start, content }, _) if content == ".." => {
          let tok = TokenEnum::Operator(Operator::Range);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Operator { content, .. }, '.') if content == "." => {
          content.push(ch);
        },
        (State::Operator { start, content }, _) if content == "." => {
          let tok = TokenEnum::Operator(Operator::Dot);

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
        (State::Operator { start, content }, _) if content == "%" => {
          let tok = TokenEnum::Operator(Operator::Modulo);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },

        (State::Operator { content, .. }, '>') if content == ">" || content == ">>" => {
          content.push(ch);
        },
        (State::Operator { content, .. }, '<') if content == "<" => {
          content.push(ch);
        },
        (State::Operator { start, content }, '=') if content == ">>>" => {
          let tok = TokenEnum::Operator(Operator::LogicalShiftRightAssign);

          add_tok(start, tok);

          state = State::Base;
        },
        (State::Operator { start, content }, _) if content == ">>>" => {
          let tok = TokenEnum::Operator(Operator::LogicalShiftRight);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Operator { start, content }, '=') if content == ">>" => {
          let tok = TokenEnum::Operator(Operator::ShiftRightAssign);

          add_tok(start, tok);

          state = State::Base;
        },
        (State::Operator { start, content }, _) if content == ">>" => {
          let tok = TokenEnum::Operator(Operator::ShiftRight);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Operator { start, content }, '=') if content == ">" => {
          let tok = TokenEnum::Operator(Operator::GreaterThanEqual);

          add_tok(start, tok);

          state = State::Base;
        },
        (State::Operator { start, content }, _) if content == ">" => {
          let tok = TokenEnum::Operator(Operator::GreaterThan);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Operator { start, content }, '=') if content == "<<" => {
          let tok = TokenEnum::Operator(Operator::ShiftLeftAssign);

          add_tok(start, tok);

          state = State::Base;
        },
        (State::Operator { start, content }, _) if content == "<<" => {
          let tok = TokenEnum::Operator(Operator::ShiftLeft);

          add_tok(start, tok);

          state = State::Base;
          continue;
        },
        (State::Operator { start, content }, '=') if content == "<" => {
          let tok = TokenEnum::Operator(Operator::LessThanEqual);

          add_tok(start, tok);

          state = State::Base;
        },
        (State::Operator { start, content }, _) if content == "<" => {
          let tok = TokenEnum::Operator(Operator::LessThan);

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
          State::NumericLiteral { content, ty: NumericType::Decimal, .. },
          '0'..='9'
        ) => {
          content.push(ch);
        },
        (State::NumericLiteral { content, ty: NumericType::Decimal, .. }, '.') if !content.contains('.') => {
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
        (State::Base, '"') => {
          state = State::StringLiteral {
            start: i,
            ty: StringType::Unicode,
            content: String::new()
          };
        },
        (State::Base, '\'') => {
          state = State::CharLiteral {
            start: i,
            ty: CharType::Unicode,
            content: String::new()
          };
        },
        (State::StringLiteral {
          start,
          content,
          ty,
        }, '"') => {
          let tok = TokenEnum::Literal({
            match ty {
              StringType::Unicode => Literal::UnicodeString(content.to_owned()),
              StringType::C => Literal::CString(content.to_owned()),
              StringType::Bytes => Literal::ByteString(content.to_owned()),
            }
          });

          add_tok(start, tok);

          state = State::Base;
        },
        (State::StringLiteral { start, ty, content }, '\\') => {
          state = State::StringEscape {
            start: *start,
            return_to: StringEscapeReturnTo::String {
              ty: *ty
            },
            content: content.to_owned(),
            ty: None
          };
        },
        (State::StringLiteral { content, .. }, _) => {
          content.push(ch);
        },
        (State::CharLiteral { start, content, ty }, '\\') => {
          state = State::StringEscape {
            start: *start,
            return_to: StringEscapeReturnTo::Char { ty: *ty },
            content: content.to_owned(),
            ty: None,
          };
        },
        (State::CharLiteral { start, ty, content }, '\'') => {
          // TODO: validate size

          let tok = TokenEnum::Literal({
            assert!(content.len() != 0);

            match ty {
              CharType::Unicode => {
                assert!(content.len() <= 4, "unicode character cannot store more than 4 bytes");
                Literal::UnicodeChar(content.chars().nth(0).unwrap())
              },
              CharType::Byte => {
                assert!(content.len() == 1, "byte char cannot store more than 1 byte");
                Literal::ByteChar(u8::try_from(content.chars().nth(0).unwrap()).unwrap())
              },
            }
          });

          add_tok(start, tok);

          state = State::Base;
        },
        (State::CharLiteral { content, .. }, _) => {
          content.push(ch);
        }
        (State::StringEscape { ty, .. }, 'x') if ty.is_none() => {
          *ty = Some(StringEscapeType::Hexadecimal { codepoint: String::new() });
        },
        (State::StringEscape { ty, .. }, 'o') if ty.is_none() => {
          *ty = Some(StringEscapeType::Octal { codepoint: String::new() });
        },
        (State::StringEscape { ty, .. }, 'u') if ty.is_none() => {
          *ty = Some(StringEscapeType::Unicode { codepoint: String::new() });
        },
        (State::StringEscape { ty: Some(StringEscapeType::Hexadecimal { codepoint }), .. }, '0'..='9' | 'a'..='f' | 'A'..='F') if codepoint.len() < 2 => {
          codepoint.push(ch);
        },
        (State::StringEscape { ty: Some(StringEscapeType::Octal { codepoint }), .. }, _) if codepoint.len() < 3 => {
          codepoint.push(ch);
        },
        (State::StringEscape { ty: Some(StringEscapeType::Unicode { codepoint }), .. }, '0'..='9' | 'a'..='f' | 'A'..='F') => {
          codepoint.push(ch);
        },
        (State::StringEscape { ty: Some(StringEscapeType::Unicode { codepoint }), .. }, '{') if codepoint.is_empty() => {
          // do nothing
        },
        (State::StringEscape {
          start,
          return_to,
          content,
          ty: Some(StringEscapeType::Hexadecimal { codepoint })
        }, _) if codepoint.len() == 2 => {
          state = State::StringEscapeFinalize {
            start: *start,
            return_to: *return_to,
            content: content.to_owned(),
            ty: Some(StringEscapeType::Hexadecimal {
              codepoint: codepoint.to_owned()
            })
          };
          continue;
        },
        (State::StringEscape {
          start,
          return_to,
          content,
          ty: Some(StringEscapeType::Octal { codepoint })
        }, _) if codepoint.len() == 3 => {
          state = State::StringEscapeFinalize {
            start: *start,
            return_to: *return_to,
            content: content.to_owned(),
            ty: Some(StringEscapeType::Octal {
              codepoint: codepoint.to_owned()
            })
          };
          continue;
        },
        (State::StringEscape {
          start,
          return_to,
          content,
          ty: Some(StringEscapeType::Unicode { codepoint })
        }, '}') => {
          state = State::StringEscapeFinalize {
            start: *start,
            return_to: *return_to,
            content: content.to_owned(),
            ty: Some(StringEscapeType::Unicode {
              codepoint: codepoint.to_owned()
            })
          };
          // don't continue -- we are consuming this ending curly brace
        },
        (State::StringEscape { start, return_to, content, ty: None }, _) => {
          content.push(match ch {
            'a' => '\x07',
            'b' => '\x08',
            't' => '\t',
            'n' => '\n',
            'v' => '\x0b',
            'f' => '\x0c',
            'r' => '\r',
            _ => ch
          });

          match return_to {
            StringEscapeReturnTo::String { ty } => {
              state = State::StringLiteral {
                start: *start,
                ty: ty.to_owned(),
                content: content.to_owned()
              };
            },
            StringEscapeReturnTo::Char { ty } => {
              state = State::CharLiteral {
                start: *start,
                ty: *ty,
                content: content.to_owned()
              };
            },
          };
        },
        (State::StringEscapeFinalize {
          start,
          return_to,
          content,
          ty: Some(
            | StringEscapeType::Hexadecimal { codepoint }
            | StringEscapeType::Octal { codepoint }
            | StringEscapeType::Unicode { codepoint }
          )
        }, _) => {
          let heuristic_start = i - codepoint.len();

          let Ok(codepoint_value) = u32::from_str_radix(&codepoint, 16) else {
            state = State::Invalid {
              start: heuristic_start,
              content: codepoint.to_owned()
            };

            break;
          };

          let Some(parsed_ch) = char::from_u32(codepoint_value) else {
            state = State::Invalid {
              start: heuristic_start,
              content: codepoint.to_owned()
            };

            break;
          };

          content.push(parsed_ch);

          match return_to {
            StringEscapeReturnTo::String { ty } => {
              state = State::StringLiteral {
                start: *start,
                ty: *ty,
                content: content.to_owned()
              };
            },
            StringEscapeReturnTo::Char { ty } => {
              state = State::CharLiteral {
                start: *start,
                ty: *ty,
                content: content.to_owned()
              };
            },
          };

          continue;
        },
        (State::Base, _) => {
          state = State::Invalid { start: i, content: String::new() };

          continue;
        },
        (State::Invalid { start, content }, '\n') => {
          add_tok(start, TokenEnum::Invalid(content.to_owned()));

          let span = toks.last().unwrap().span.to_owned();

          return InvalidSourceSnafu {
            parsed: toks,
            span
          }.fail();
        },
        (State::Invalid { content, .. }, _) => {
          content.push(ch);
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
