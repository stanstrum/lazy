use std::collections::VecDeque;

use crate::Result;
use crate::tokenizer::SpanStart;

#[derive(Debug, Clone, Copy)]
pub(super) struct ReaderItem {
  pub position: usize,
  pub ch: char,
}

pub(super) struct PeekReader<'a> {
  reader: &'a mut dyn Iterator<Item = Result<ReaderItem>>,
  buffer: VecDeque<ReaderItem>,
  pub position: usize,
}

impl<'a> Iterator for PeekReader<'a> {
  type Item = Result<ReaderItem>;

  fn next(&mut self) -> Option<Self::Item> {
    self.position += 1;

    if let Some(buffered) = self.buffer.pop_front() {
      return Some(Ok(buffered));
    };

    self.reader.next()
  }
}

impl<'a> PeekReader<'a> {
  pub(super) fn new(reader: &'a mut dyn Iterator<Item = Result<ReaderItem>>) -> Self {
    Self {
      reader,
      buffer: VecDeque::new(),
      position: 0,
    }
  }

  pub(super) fn span_start(&mut self) -> SpanStart {
    SpanStart(self.position)
  }

  pub(super) fn seek(&mut self) {
    if self.buffer.pop_front().is_some() {
      return;
    };

    self.next();
  }

  // TODO: i don't like this
  pub(super) fn seek_n(&mut self, count: usize) {
    for _ in 0..count {
      self.seek();
    };
  }

  pub(super) fn peek(&mut self) -> Result<Option<ReaderItem>> {
    if let Some(item) = self.buffer.front().cloned() {
      return Ok(Some(item));
    };

    let Some(item) = self.reader.next() else {
      return Ok(None);
    };

    let item = item?;
    self.buffer.push_back(item);

    Ok(Some(item))
  }

  fn peek_take(&mut self, count: usize) -> Result<Option<impl Iterator<Item = char> + '_>> {
    assert!(count != 0, "peek must be of non-zero length (for span data)");

    if self.buffer.len() < count {
      let needed = count - self.buffer.len();

      for _ in 0..needed {
        let Some(item) = self.reader.next() else {
          return Ok(None);
        };

        self.buffer.push_back(item?);
      };
    };

    let iter = self.buffer
      .iter()
      .take(count)
      .map(|item| item.ch);

    Ok(Some(iter))
  }

  pub(super) fn starts_with(&mut self, text: &str) -> Result<bool> {
    let Some(peek) = self.peek_take(text.len())? else {
      return Ok(false);
    };

    for (a, b) in peek.zip(text.chars()) {
      if a != b {
        return Ok(false);
      };
    };

    Ok(true)
  }

  pub(super) fn starts_with_seek(&mut self, text: &str) -> Result<bool> {
    let result = self.starts_with(text)?;

    if result {
      self.seek_n(text.len());
    };

    Ok(result)
  }
}
