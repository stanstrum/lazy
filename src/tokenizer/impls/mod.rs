pub(crate) mod whitespace;
pub(crate) mod comment;
pub(crate) mod ident;
pub(crate) mod operator;
pub(crate) mod numeric;

use crate::compiler::Result;
use crate::tokenizer::{
  PeekReader,
  TokenKind,
  Grouping,
};

impl crate::tokenizer::Tokenizer {
  pub(in crate::tokenizer) fn base(&mut self, reader: &mut PeekReader) -> Result {
    trace!("Tokenizer::base");
    let start = reader.span_start();

    let Some(item) = reader.peek()? else {
      return Ok(());
    };

    if let Some(grouping) = Grouping::from_str(&String::from(item.ch)) {
      reader.seek();
      self.push_tok(TokenKind::Grouping(grouping), start, reader.position);
      return Ok(());
    };

    match item.ch {
      whitespace!() => self.whitespace(reader),
      ident!() => self.identifier(reader),
      operator!() => self.operator(reader),
      decimal!() => self.numeric(reader),
      _ => todo!("{:?}", item.ch),
    }
  }
}
