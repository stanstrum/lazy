use crate::compiler::CompilerResult;
use crate::tokenizer::{
  PeekReader,
  Tokenizer,
  TokenKind,
  SpanStart,
};

impl Tokenizer {
  pub(in crate::tokenizer) fn whitespace(&mut self, reader: &mut PeekReader) -> CompilerResult<()> {
    trace!("Tokenizer::whitespace");

    let Some(item) = reader.next() else {
      return Ok(());
    };

    let item = item?;

    let start = SpanStart(item.position);
    let mut end = item.position;

    while let Some(item) = reader.peek()? {
      let whitespace!() = item.ch else {
        break;
      };

      end = item.position;
      reader.seek();
    };

    self.push_tok(TokenKind::Whitespace, start, end);

    Ok(())
  }
}
