use crate::compiler::CompilerResult;
use crate::tokenizer::{
  PeekReader,
  Tokenizer,
  TokenKind,
  Keyword,
  SpanStart,
};

impl Tokenizer {
  pub(in crate::tokenizer) fn identifier(&mut self, reader: &mut PeekReader) -> CompilerResult<()> {
    trace!("Tokenizer::identifier");

    let Some(item) = reader.next() else {
      return Err("expected an identifier".into());
    };
    let item = item?;

    let ident!() = item.ch else {
      return Err(format!("expected an identifier: {:?}", item.ch))
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
