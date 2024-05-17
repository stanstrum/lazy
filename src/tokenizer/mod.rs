mod structs;
mod to_string;

pub(crate) mod error;

use std::fs::File;
use utf8_read::Reader;

mod state;
use state::*;

pub(crate) use structs::*;
pub(crate) use error::TokenizationError;

pub(crate) fn stringify(tokens: &Vec<Token>) -> String {
  let mut source = String::new();

  for token in tokens.iter() {
    source += token.to_string().as_str();
  };

  source
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
        (State::Text { start, content }, '"') if content == "b" => {
          state = State::StringLiteral {
            start: *start,
            escape_next: false,
            ty: StringType::Bytes,
            content: String::new()
          };
        },
        (State::Text { start, content }, '"') if content == "c" => {
          state = State::StringLiteral {
            start: *start,
            escape_next: false,
            ty: StringType::C,
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
        (State::Operator { start, content }, '.') if content == "." => {
          let tok = TokenEnum::Operator(Operator::Range);

          add_tok(start, tok);

          state = State::Base;
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
            escape_next: false,
            ty: StringType::Unicode,
            content: String::new()
          };
        },
        (State::StringLiteral { escape_next, .. }, '\\') if !*escape_next => {
          *escape_next = true;
        },
        (State::StringLiteral { content, escape_next, .. }, '\\') if *escape_next => {
          content.push('\\');
          *escape_next = false;
        },
        (State::StringLiteral { content, escape_next, .. }, 'n') if *escape_next => {
          content.push('\n');
          *escape_next = false;
        },
        (State::StringLiteral { content, escape_next, .. }, 'r') if *escape_next => {
          content.push('\r');
          *escape_next = false;
        },
        (State::StringLiteral { content, escape_next, .. }, 't') if *escape_next => {
          content.push('\t');
          *escape_next = false;
        },
        (State::StringLiteral { content, escape_next, .. }, '0') if *escape_next => {
          content.push('\0');
          *escape_next = false;
        },
        // TODO: add more escape codes
        (State::StringLiteral {
          start,
          content,
          ty,
          escape_next: false
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
        (State::StringLiteral { content, .. }, _) => {
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
