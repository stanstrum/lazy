use std::io::Read;

use lang::Compiler;
use lang::span::Span;

use crate::token::{
  CharState,
  EscapeReturn,
  EscapeValue,
  GroupingKind,
  GroupingType,
  Keyword,
  NumericKind,
  Operator,
  Position,
  StringKind,
  StringState,
  Token,
  TokenSpan,
  parse_escape,
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
  String(StringState),
  Char(CharState),
  Escape {
    content: String,
    ret: EscapeReturn
  },
}

impl<'pool, C: Compiler, const N: usize, T: Read> Tokenizer<'pool, C, N, T> {
  pub(super) fn do_state(&mut self) -> Option<Result<TokenSpan<C>, Error<C>>> {
    loop {
      if let Some(tok) = self.toks.pop_front() {
        return Some(Ok(tok));
      };

      let ch = match self.take_ch() {
        Ok(Some(ch)) => ch,
        Ok(None) => return None,
        Err(_) => return Some(Err(Error::IO {
          module: self.module,
          name: self.name.to_owned(),
        })),
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
        (State::Base, '-' | '/' | ':' | ';' | '&' | '+' | '*' | '%' | '|' | '^' | '?' | '.' | '>' | '<' | '=' | '~' | '!' | ',') => {
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
          let indentation = self.override_indentation.take()
            .unwrap_or(self.meta_reader.meta.whitespace) as isize;

          loop {
            let result = self.meta_reader.next()?;

            let Ok(ch) = result else {
              return Some(Err(Error::IO {
                module: self.module,
                name: self.name.to_owned(),
              }));
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
        // -> String
        (State::Text { content, start }, '"') if matches!(content.as_str(), "b" | "c") => {
          self.state = State::String(StringState {
            content: String::new(),
            kind: match content.as_str() {
              "b" => StringKind::Byte,
              "c" => StringKind::C,
              _ => unimplemented!(),
            },
            start: *start,
          });
        },
        // -> Char
        (State::Text { content, .. }, '\'') if content == "b" => {
          todo!("{content}-char parse state")
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
            (_, _) if content.is_empty()
              => content.push(ch),
            // comments
            | ("", '/')
            | ("/", '/') // //
            | ("/", '*') // /*
            // operators
            | ("-", '>') // ->
            | (":", '=') // :=
            | (":", ':') //
            // arithmetic
            | ("+", '+' | '=') // ++ and +=
            | ("-", '-' | '=') // -- and -=
            | ("*", '*' | '=') // ** and *=
            | ("**", '=') // **=
            | ("/", '=') // /=
            | ("%", '=') // %=
            // bit
            | ("&", '&' | '=') // && and &=
            | ("&&", '=') // &&=
            | ("|", '|' | '=') // || and |=
            | ("||", '=') // ||=
            | ("^", '^' | '=') // ^^ and ^=
            | ("^^", '=') // ^^=
            | ("<", '<' | '=') // <= and <<
            | ("<<", '=') // <<=
            | (">", '>' | '=') // >= and >>
            | (">>", '>' | '=') // >>= and >>>
            | (">>>", '=') // >>>=
            | ("=", '=') // ==
            //
            | ("." | "..", '.') // .. and ...
              => content.push(ch),
            ("->", _) => {
              self.push_here(Token::Operator(Operator::RightArrow), start);
              self.retry(ch, State::Base);
            },
            (":=", _) => {
              self.push_here(Token::Operator(Operator::Bollocks), start);
              self.retry(ch, State::Base);
            },
            ("::", _) => {
              self.push_here(Token::Operator(Operator::DoubleColon), start);
              self.retry(ch, State::Base);
            }
            (";", _) => {
              self.push_here(Token::Operator(Operator::Semicolon), start);
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
            },
            // Math
            ("+", _) => {
              self.push_here(Token::Operator(Operator::Plus), start);
              self.retry(ch, State::Base);
            },
            ("-", _) => {
              self.push_here(Token::Operator(Operator::Minus), start);
              self.retry(ch, State::Base);
            },
            ("*", _) => {
              self.push_here(Token::Operator(Operator::Asterisk), start);
              self.retry(ch, State::Base);
            },
            ("/", _) => {
              self.push_here(Token::Operator(Operator::Div), start);
              self.retry(ch, State::Base);
            },
            ("%", _) => {
              self.push_here(Token::Operator(Operator::Mod), start);
              self.retry(ch, State::Base);
            },
            ("**", _) => {
              self.push_here(Token::Operator(Operator::Asterisk), start);
              self.push_here(Token::Operator(Operator::Asterisk), start);
              self.retry(ch, State::Base);
            },
            ("+=", _) => {
              self.push_here(Token::Operator(Operator::AddAssign), start);
              self.retry(ch, State::Base);
            },
            ("-=", _) => {
              self.push_here(Token::Operator(Operator::SubAssign), start);
              self.retry(ch, State::Base);
            },
            ("*=", _) => {
              self.push_here(Token::Operator(Operator::MulAssign), start);
              self.retry(ch, State::Base);
            },
            ("/=", _) => {
              self.push_here(Token::Operator(Operator::DivAssign), start);
              self.retry(ch, State::Base);
            },
            ("%=", _) => {
              self.push_here(Token::Operator(Operator::ModAssign), start);
              self.retry(ch, State::Base);
            },
            ("**=", _) => {
              self.push_here(Token::Operator(Operator::ExpAssign), start);
              self.retry(ch, State::Base);
            },
            ("|", _) => {
              self.push_here(Token::Operator(Operator::Or), start);
              self.retry(ch, State::Base);
            },
            ("&", _) => {
              self.push_here(Token::Operator(Operator::SingleAnd), start);
              self.retry(ch, State::Base);
            },
            ("^", _) => {
              self.push_here(Token::Operator(Operator::Xor), start);
              self.retry(ch, State::Base);
            },
            ("|=", _) => {
              self.push_here(Token::Operator(Operator::OrAssign), start);
              self.retry(ch, State::Base);
            },
            ("&=", _) => {
              self.push_here(Token::Operator(Operator::AndAssign), start);
              self.retry(ch, State::Base);
            },
            ("^=", _) => {
              self.push_here(Token::Operator(Operator::XorAssign), start);
              self.retry(ch, State::Base);
            },
            ("||", _) => {
              self.push_here(Token::Operator(Operator::LogicalOr), start);
              self.retry(ch, State::Base);
            },
            ("&&", _) => {
              self.push_here(Token::Operator(Operator::LogicalAnd), start);
              self.retry(ch, State::Base);
            },
            ("^^", _) => {
              self.push_here(Token::Operator(Operator::LogicalXor), start);
              self.retry(ch, State::Base);
            },
            ("||=", _) => {
              self.push_here(Token::Operator(Operator::LogicalOrAssign), start);
              self.retry(ch, State::Base);
            },
            ("&&=", _) => {
              self.push_here(Token::Operator(Operator::LogicalAndAssign), start);
              self.retry(ch, State::Base);
            },
            ("^^=", _) => {
              self.push_here(Token::Operator(Operator::LogicalXorAssign), start);
              self.retry(ch, State::Base);
            },
            ("++", _) => {
              self.push_here(Token::Operator(Operator::DoublePlus), start);
              self.retry(ch, State::Base);
            },
            ("--", _) => {
              self.push_here(Token::Operator(Operator::DoubleMinus), start);
              self.retry(ch, State::Base);
            },
            (".", _) => {
              self.push_here(Token::Operator(Operator::Dot), start);
              self.retry(ch, State::Base);
            },
            ("..", _) => {
              self.push_here(Token::Operator(Operator::Range), start);
              self.retry(ch, State::Base);
            },
            ("...", _) => {
              self.push_here(Token::Operator(Operator::Splat), start);
              self.retry(ch, State::Base);
            },
            (":", _) => {
              self.push_here(Token::Operator(Operator::Colon), start);
              self.retry(ch, State::Base);
            },
            (",", _) => {
              self.push_here(Token::Operator(Operator::Comma), start);
              self.retry(ch, State::Base);
            },
            ("<<=", _) => {
              self.push_here(Token::Operator(Operator::ShlAssign), start);
              self.retry(ch, State::Base);
            },
            ("<<", _) => {
              self.push_here(Token::Operator(Operator::Shl), start);
              self.retry(ch, State::Base);
            },
            ("<=", _) => {
              self.push_here(Token::Operator(Operator::LessEqual), start);
              self.retry(ch, State::Base);
            },
            ("<", _) => {
              self.push_here(Token::Operator(Operator::Less), start);
              self.retry(ch, State::Base);
            },
            (">>>=", _) => {
              self.push_here(Token::Operator(Operator::LogicalShrAssign), start);
              self.retry(ch, State::Base);
            },
            (">>>", _) => {
              self.push_here(Token::Operator(Operator::LogicalShr), start);
              self.retry(ch, State::Base);
            },
            (">>=", _) => {
              self.push_here(Token::Operator(Operator::ShrAssign), start);
              self.retry(ch, State::Base);
            },
            (">>", _) => {
              self.push_here(Token::Operator(Operator::Shr), start);
              self.retry(ch, State::Base);
            },
            (">=", _) => {
              self.push_here(Token::Operator(Operator::GreaterEqual), start);
              self.retry(ch, State::Base);
            },
            (">", _) => {
              self.push_here(Token::Operator(Operator::Greater), start);
              self.retry(ch, State::Base);
            },
            ("==", _) => {
              self.push_here(Token::Operator(Operator::Equal), start);
              self.retry(ch, State::Base);
            },
            ("=", _) => {
              self.push_here(Token::Operator(Operator::Assign), start);
              self.retry(ch, State::Base);
            },
            ("!", _) => {
              self.push_here(Token::Operator(Operator::Not), start);
              self.retry(ch, State::Base);
            },
            ("~", _) => {
              self.push_here(Token::Operator(Operator::Invert), start);
              self.retry(ch, State::Base);
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

          trim_in_place(&mut content);

          let comment_id = self.pool.insert_string(content);
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

          let comment_id = self.pool.insert_string(content);

          self.override_indentation.get_or_insert(start.indentation);
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

          let temp_span = Span {
            start,
            end: self.pos(),
            module: self.module,
          };

          let tok = match self.parse_and_push(temp_span, kind, &content) {
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

          let temp_span = Span {
            start,
            end: self.pos(),
            module: self.module,
          };

          let tok = match self.parse_and_push(temp_span, kind, &content) {
            Ok(tok) => tok,
            Err(error) => return Some(Err(error)),
          };

          self.push_here(tok, start);
          self.save(ch);
        },
        // String
        (State::Base, '"') => {
          self.state = State::String(StringState {
            content: String::new(),
            kind: StringKind::Wide,
            start: self.pos(),
          });
        },
        (State::String(_), '"') => {
          let State::String(string) = std::mem::replace(&mut self.state, State::Base) else {
            unreachable!();
          };

          let id = self.pool.insert_string(string.content);
          let end = match self.next_pos() {
            Ok(end) => end,
            Err(err) => return Some(Err(err)),
          };

          let span = Span {
            start: string.start,
            end,
            module: self.module,
          };

          self.toks.push_back((Token::String(string.kind, id), span));
        },
        (State::String(_), '\\') => {
          let State::String(string) = std::mem::replace(&mut self.state, State::Base) else {
            unreachable!();
          };

          self.state = State::Escape {
            content: String::new(),
            ret: EscapeReturn::String(string),
          };
        },
        (State::String(StringState { content, .. }), _) => {
          content.push(ch);
        },
        // Escape
        (State::Escape { content, ret }, _) => {
          content.push(ch);

          // dbg!(ch);

          match parse_escape(content) {
            Ok(EscapeValue::Char(ch)) => {
              ret.append_ch(ch);

              let State::Escape { ret, .. } = std::mem::replace(&mut self.state, State::Base) else { unreachable!() };
              self.state = ret.into();
            },
            Ok(value) => todo!("{value:?}"),
            Err(err) => return Some(Err(err)),
          };
        },
        // Fallthrough
        other => todo!("tokenize state {other:?}"),
      }
    }
  }
}
