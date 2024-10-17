use crate::ok;

use crate::Result;
use crate::tokenizer::{
  PeekReader,
  Tokenizer,
  TokenKind,
};

impl Tokenizer {
  pub(in crate::tokenizer) fn line_comment(&mut self, reader: &mut PeekReader) -> Result {
    trace!("Tokenizer::line_comment");
    let mut message = String::new();
    let start = reader.span_start();

    for item in &mut *reader {
      let item = item?;

      if let '\n' = item.ch {
        break;
      };

      message.push(item.ch);
    };

    self.push_tok(TokenKind::Comment(message.trim().into()), start, reader.position);

    ok
  }
}
