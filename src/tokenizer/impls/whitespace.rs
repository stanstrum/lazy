use crate::compiler::CompilerWorkflow;
use crate::{Result, ok};
use crate::tokenizer::{
  PeekReader,
  Tokenizer,
  TokenKind,
  SpanStart,
};

impl<W: CompilerWorkflow> Tokenizer<W> {
  pub(in crate::tokenizer) fn whitespace(&mut self, reader: &mut PeekReader) -> Result {
    trace!("Tokenizer::whitespace");

    let Some(item) = reader.next() else {
      return ok;
    };

    let item = item?;

    let start = SpanStart {
      start: item.position,
      handle: todo!(),
      marker: Default::default(),
    };
    let mut end = item.position;

    while let Some(item) = reader.peek()? {
      let whitespace!() = item.ch else {
        break;
      };

      end = item.position;
      reader.seek();
    };

    self.push_tok(TokenKind::Whitespace, start, end);

    ok
  }
}
