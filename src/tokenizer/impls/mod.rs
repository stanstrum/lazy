mod whitespace;
mod comment;
mod ident;
mod operator;
pub(crate) mod numeric;
mod string;

use crate::{Result, ok};
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
      return ok;
    };

    if let Some(grouping) = Grouping::from_str(String::from(item.ch).as_str()) {
      reader.seek();
      self.push_tok(TokenKind::Grouping(grouping), start, reader.position);
      return ok;
    };

    match item.ch {
      whitespace!() => self.whitespace(reader),
      ident!() => self.identifier(reader),
      operator!() => self.operator(reader),
      decimal!() => self.numeric(reader),
      '"' => self.string(reader),
      '\'' => self.char(reader),
      _ => todo!("{:?}", item.ch),
    }
  }
}
