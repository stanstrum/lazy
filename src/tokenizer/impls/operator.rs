use crate::compiler::Result;
use crate::tokenizer::{
  PeekReader,
  Tokenizer,
  TokenKind,
  Operator,
  Punctuation,
  SpanStart,
  error::*,
};

impl Tokenizer {
  pub(in crate::tokenizer) fn operator(&mut self, reader: &mut PeekReader) -> Result {
    trace!("Tokenizer::operator");

    let Some(item) = reader.peek()? else {
      return ExpectedSnafu { what: What::Operator }.fail()?;
    };

    let start = SpanStart(item.position);
    let mut end = start.0;

    let mut content = String::new();

    loop {
      let Some(item) = reader.next() else {
        break;
      };
      let item = item?;

      end = item.position;
      content.push(item.ch);

      let Some(peek) = reader.peek()? else {
        break;
      };

      match (content.as_str(), peek.ch) {
        | ("%", '=')
        | ("^", '^' | '=')
        | ("^^", '=')
        | ("&", '&' | '=')
        | ("&&", '=')
        | ("*", '*' | '=')
        | ("**", '=')
        | ("-", '=' | '-' | '>')
        | ("+", '=' | '+')
        | ("=", '=')
        | ("|", '|' | '=')
        | ("||", '|' | '=')
        | ("<", '<' | '=')
        | ("<<", '<' | '=')
        | ("<<<", '=')
        | (">", '>' | '=')
        | (">>", '>' | '=')
        | (">>>", '=')
        | ("/", '/' | '*' | '=')
        | (":", ':')
        | (".", '.')
        | ("..", '.')
        => {},
        ("//", _) => {
          self.line_comment(reader)?;
          return Ok(());
        },
        ("/*", _) => todo!("multiline comment"),
        _ => break,
      };
    };

    let kind = if let Some(op) = Operator::from_str(&content) {
      TokenKind::Operator(op)
    } else if let Some(punct) = Punctuation::from_str(&content) {
      TokenKind::Punctuation(punct)
    } else {
      // TODO: `unrecognized` isn't strictly the same as `expected`

      return ExpectedSnafu { what: What::Operator }.fail()?;
      // return OtherSnafu { err: format!("unrecognized operator: {content:?}") }.fail()?;
    };

    self.push_tok(kind, start, end);

    Ok(())
  }
}
