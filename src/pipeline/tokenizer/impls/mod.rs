mod comment;
mod ident;
pub(crate) mod numeric;
mod operator;
mod string;
mod whitespace;

use crate::compiler::CompilerWorkflow;
use crate::tokenizer::{Grouping, PeekReader, TokenKind, Tokenizer};
use crate::{ok, Result};

impl<W: CompilerWorkflow> Tokenizer<W> {
  pub(in crate::pipeline::tokenizer) fn base(&mut self, reader: &mut PeekReader<W>) -> Result {
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
