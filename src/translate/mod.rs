mod mark_reader;

use super::*;
use crate::token::{SourceMark, Span, Token, TokenKind};

pub(super) struct Translator<I: Iterator<Item = Token>> {
  id: usize,
  iter: I,
  index: usize,
  buffer: Vec<Token>,
  tx: Sender<CompilerSignal>,
  mark: SourceMark,
}

pub(super) trait Translate {
  fn translate<I: Iterator<Item = Token>>(
    translator: &mut Translator<I>,
  ) -> Result<Option<()>, Whatever>;
}

impl<I: Iterator<Item = Token>> Translator<I> {
  pub(super) fn skip_whitespace(&mut self) {
    while let Some((TokenKind::Whitespace(_), _)) = self.peek() {
      self.seek();
    }
  }
}

impl Translate for lang::Function {
  fn translate<I: Iterator<Item = Token>>(
    translator: &mut Translator<I>,
  ) -> Result<Option<()>, Whatever> {
    let Some((TokenKind::Identifier(name), Span { start, .. })) = translator.peek() else {
      return Ok(None);
    };

    translator.seek();
    translator.skip_whitespace();

    todo!()
  }
}
