use crate::tokenizer::{
  PeekReader,
  Tokenizer,
  TokenKind,
  Operator,
  Punctuation,
  SpanStart,
};
use crate::compiler::CompilerResult;

impl Tokenizer {
  pub(in crate::tokenizer) fn operator(&mut self, reader: &mut PeekReader) -> CompilerResult<()> {
    trace!("Tokenizer::operator");
    let Some(item) = reader.peek()? else {
      return Err("expected an operator".into());
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
      return Err(format!("unrecognized operator: {content:?}"));
    };

    self.push_tok(kind, start, end);

    Ok(())
  }
}
