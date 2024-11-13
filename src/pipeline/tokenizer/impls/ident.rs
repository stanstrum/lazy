use crate::compiler::CompilerWorkflow;
use crate::tokenizer::{error::*, Keyword, PeekReader, SpanStart, TokenKind, Tokenizer};
use crate::{ok, Result};

impl<W: CompilerWorkflow> Tokenizer<W> {
  pub(in crate::pipeline::tokenizer) fn identifier(
    &mut self,
    reader: &mut PeekReader<W>,
  ) -> Result {
    let Some(item) = reader.next() else {
      return ExpectedSnafu {
        what: What::Identifier,
      }
      .fail()?;
    };
    let item = item?;

    let ident!() = item.ch else {
      return ExpectedSnafu {
        what: What::Identifier,
      }
      .fail()?;
    };

    let start = SpanStart {
      start: item.position,
      handle: reader.handle,
    };
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
    }

    let end = start.start + name.len();

    let kind = if let Some(keyword) = Keyword::from_str(&name) {
      TokenKind::Keyword(keyword)
    } else {
      TokenKind::Identifier(name)
    };

    self.push_tok(kind, start, end);

    ok
  }
}
