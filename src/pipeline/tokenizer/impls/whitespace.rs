use crate::compiler::CompilerWorkflow;
use crate::tokenizer::{PeekReader, SpanStart, TokenKind, Tokenizer};
use crate::{ok, Result};

impl<W: CompilerWorkflow> Tokenizer<W> {
  pub(in crate::pipeline::tokenizer) fn whitespace(
    &mut self,
    reader: &mut PeekReader<W>,
  ) -> Result {
    let Some(item) = reader.next() else {
      return ok;
    };

    let item = item?;

    let start = SpanStart {
      start: item.position,
      handle: reader.handle,
    };
    let mut end = item.position;

    while let Some(item) = reader.peek()? {
      let whitespace!() = item.ch else {
        break;
      };

      end = item.position;
      reader.seek();
    }

    self.push_tok(TokenKind::Whitespace, start, end);

    ok
  }
}
