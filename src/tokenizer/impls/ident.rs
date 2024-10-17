use crate::compiler::Result;
use crate::tokenizer::{
  PeekReader,
  Tokenizer,
  TokenKind,
  Keyword,
  SpanStart,
  error::*,
};

impl Tokenizer {
  pub(in crate::tokenizer) fn identifier(&mut self, reader: &mut PeekReader) -> Result {
    trace!("Tokenizer::identifier");

    let Some(item) = reader.next() else {
      return ExpectedSnafu { what: What::Identifier }.fail()?;
    };
    let item = item?;

    let ident!() = item.ch else {
      return ExpectedSnafu { what: What::Identifier }.fail()?;
    };

    let start = SpanStart(item.position);
    let mut name = String::from(item.ch);

    loop {
      let Some(peek) = reader.peek()? else {
        break;
      };

      let (ident!() | decimal!()) = peek.ch else {
        break;
      };

      name.push(peek.ch);
      reader.seek();
    };

    let kind = if let Some(keyword) = Keyword::from_str(&name) {
      TokenKind::Keyword(keyword)
    } else {
      TokenKind::Identifier(name)
    };

    self.push_tok(kind, start, reader.position);

    Ok(())
  }

}
