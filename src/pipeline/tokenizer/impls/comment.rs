use crate::ok;

use crate::Result;
use crate::tokenizer::{
  PeekReader,
  Tokenizer,
  TokenKind,
  error::*,
};
use crate::compiler::CompilerWorkflow;

impl<W: CompilerWorkflow> Tokenizer<W> {
  pub(in crate::pipeline::tokenizer) fn line_comment(&mut self, reader: &mut PeekReader<W>) -> Result {
    let mut message = String::new();
    let start = reader.span_start();

    for item in &mut *reader {
      let item = item?;

      if let '\n' = item.ch {
        break;
      };

      message.push(item.ch);
    };

    let kind = TokenKind::Comment(message.trim().into());

    self.push_tok(kind, start, reader.position);

    ok
  }

  pub(in crate::pipeline::tokenizer) fn multiline_comment(&mut self, reader: &mut PeekReader<W>) -> Result {
    const COMMENT_OPEN: &str = "/*";
    const COMMENT_CLOSE: &str = "*/";

    let mut level = 1;
    let start = reader.span_start();

    let mut content = String::new();

    loop {
      if reader.starts_with_seek(COMMENT_OPEN)? {
        level += 1;

        if level != 0 {
          content.push_str(COMMENT_OPEN);
        };

        continue;
      };

      if reader.starts_with_seek(COMMENT_CLOSE)? {
        level -= 1;

        if level == 0 {
          break;
        };

        content.push_str(COMMENT_CLOSE);
        continue;
      };

      let Some(item) = reader.next() else {
        return InvalidSnafu {
          what: What::MultilineComment,
          content,
        }.fail()?;
      };

      content.push(item?.ch);
    };

    self.push_tok(TokenKind::Comment(content), start, reader.position);

    ok
  }
}
