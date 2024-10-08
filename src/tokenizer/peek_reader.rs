use crate::compiler::CompilerResult;
use crate::tokenizer::SpanStart;

#[derive(Debug, Clone, Copy)]
pub(super) struct ReaderItem {
  pub position: usize,
  pub ch: char,
}

pub(super) struct PeekReader<'a> {
  reader: &'a mut dyn Iterator<Item = CompilerResult<ReaderItem>>,
  peek_buffer: Option<ReaderItem>,
  pub position: usize,
}

impl<'a> PeekReader<'a> {
  pub(super) fn new(reader: &'a mut dyn Iterator<Item = CompilerResult<ReaderItem>>) -> Self {
    Self {
      reader,
      peek_buffer: None,
      position: 0,
    }
  }

  pub(super) fn seek(&mut self) {
    if self.peek_buffer.is_some() {
      self.peek_buffer = None;
    } else {
      self.next();
    };
  }

  pub(super) fn peek(&mut self) -> CompilerResult<Option<ReaderItem>> {
    if let Some(buffered) = self.peek_buffer {
      return Ok(Some(buffered));
    };

    let Some(item) = self.next() else {
      return Ok(None);
    };

    let item = item?;
    self.peek_buffer = Some(item);

    Ok(Some(item))
  }

  pub(super) fn span_start(&self) -> SpanStart {
    SpanStart(self.position)
  }
}

impl Iterator for PeekReader<'_> {
  type Item = CompilerResult<ReaderItem>;

  fn next(&mut self) -> Option<Self::Item> {
    let (message, result) = {
      if let Some(buffered) = self.peek_buffer {
        self.peek_buffer = None;

        ("buffered", Some(Ok(buffered)))
      } else {
        let next = self.reader.next();

        if next.as_ref().is_some_and(|next| next.is_ok()) {
          self.position += 1;
        };

        ("read", next)
      }
    };

    trace!("PeekReader::next {message}   \t-> {result:?}");
    result
  }
}
