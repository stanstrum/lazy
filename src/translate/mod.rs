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

impl Translate for lang::Expression {
  fn translate<I: Iterator<Item = Token>>(
    translator: &mut Translator<I>,
  ) -> Result<Option<()>, Whatever> {
    todo!()
  }
}

impl Translate for lang::Block {
  fn translate<I: Iterator<Item = Token>>(
    translator: &mut Translator<I>,
  ) -> Result<Option<()>, Whatever> {
    let start = translator.mark;

    'harness: {
      let Some((
        TokenKind::Grouping(token::Grouping::Open(token::GroupingKind::Curly)),
        Span { start, .. },
      )) = translator.peek()
      else {
        break 'harness;
      };
      translator.seek();

      loop {
        translator.skip_whitespace();

        if let Some((TokenKind::Grouping(token::Grouping::Close(token::GroupingKind::Curly)), _)) =
          translator.peek()
        {
          translator.seek();
          break;
        };

        if lang::Expression::translate(translator)?.is_none() {
          eprintln!("[translate] skipping until next line");
          // let line = translator.mark;

          todo!("skip til next line and continue parsing")
        };

        translator.skip_whitespace();

        let Some((kind, span)) = translator.next() else {
          break 'harness;
        };

        match kind {
          TokenKind::Grouping(token::Grouping::Close(token::GroupingKind::Curly)) => break,
          TokenKind::Punctuation(token::Punctuation::Semicolon) => continue,
          _ => panic!("error: expected close curly or semicolon"),
        }
      }

      return Ok(Some(()));
    }

    translator.pop(start);
    Ok(None)
  }
}

impl Translate for lang::Function {
  fn translate<I: Iterator<Item = Token>>(
    translator: &mut Translator<I>,
  ) -> Result<Option<()>, Whatever> {
    let start = translator.mark;

    'harness: {
      let Some((TokenKind::Identifier(name), _)) = translator.next() else {
        break 'harness;
      };

      let name = name.to_owned();

      translator.seek();
      translator.skip_whitespace();

      // let Some(next) = translator.peek() else {
      //   break 'harness;
      // };

      println!("stub: parse arguments & return type for fn ast");

      let Some(()) = lang::Block::translate(translator)? else {
        break 'harness;
      };

      println!("[translate] found a function: {name} -- stub registration");

      return Ok(Some(()));
    };

    translator.pop(start);
    Ok(None)
  }
}

impl<I: Iterator<Item = Token>> Translator<I> {
  fn translate<T: Translate>(&mut self) -> Result<Option<()>, Whatever> {
    todo!()
  }
}
