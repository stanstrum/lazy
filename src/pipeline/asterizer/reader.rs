
use crate::whitespace_or_comment;

use crate::compiler::{
  CompilerWorkflow,
  CompilerStoreHandle,
};
use crate::tokenizer::{
  Span,
  SpanStart,
  Token,
  TokenKind,
};

/// A reader for Tokens that allows for peeking, reading, setting, and resetting
/// the internal reader position at will
#[derive(Debug)]
pub(super) struct TokenReader<W: CompilerWorkflow> {
  /// The tokens taken from the tokenization stage
  tokens: Vec<Token<W>>,
  /// The handle of the module being processed
  handle: CompilerStoreHandle<W>,
  /// The saved position markers that can be used to restore the internal reader
  /// position
  marks: Vec<usize>,
  /// The current position of the reader along the Token vector
  position: usize,
}

impl<W: CompilerWorkflow> TokenReader<W> {
  /// Creates a TokenReader provided Tokens
  pub(super) fn new(tokens: Vec<Token<W>>, handle: CompilerStoreHandle<W>) -> Self {
    Self {
      tokens,
      handle,
      marks: vec![],
      position: 0,
    }
  }

  /// Pushes a mark that can later be popped to return to a certain starting
  /// point
  pub(super) fn push_mark(&mut self) {
    self.marks.push(self.position);
  }

  /// Pops a mark and restores the internal reader position to the last position
  /// saved
  pub(super) fn pop_mark(&mut self) {
    let Some(mark) = self.marks.pop() else {
      panic!("tried to pop mark with none present");
    };

    self.position = mark;
  }

  /// Drops a mark previously pushed without changing the reader's current
  /// position
  pub(super) fn drop_mark(&mut self) {
    let Some(_) = self.marks.pop() else {
      panic!("tried to drop mark with none present");
    };
  }

  /// Advances the reader position
  pub(super) fn seek(&mut self) {
    self.position += 1;
  }

  /// Peeks the next Token without advancing the reader position
  pub(super) fn peek(&self) -> Option<&Token<W>> {
    self.tokens.get(self.position)
  }

  /// Peeks the next TokenKind without advancing the reader position
  #[allow(unused)]
  pub(super) fn peek_kind(&self) -> Option<&TokenKind> {
    self.peek().map(|tok| &tok.kind)
  }

  /// Reads the next Token and advances the reader position
  pub(super) fn next(&mut self) -> Option<&Token<W>> {
    let tok = self.tokens.get(self.position);
    self.position += 1;

    tok
  }

  /// Reads the next TokenKind and advances the reader position
  pub(super) fn next_kind(&mut self) -> Option<&TokenKind> {
    self.next().map(|tok| &tok.kind)
  }

  /// Gets the current position in the source code
  pub(super) fn get_position(&self) -> usize {
    if let Some(peek) = self.peek() {
      // Either the next Token, ...
      peek.span.start
    } else if let Some(last) = self.tokens.last() {
      // Last Token, ...
      last.span.start
    } else {
      // Or just 0, representing the beginning of an empty file since there is
      // no last Token
      0
    }
  }

  /// Computes a SpanStart that refers to the next Token to be read
  pub(super) fn get_start(&self) -> SpanStart<W> {
    SpanStart {
      start: self.get_position(),
      handle: self.handle,
    }
  }

  /// Whether all the Tokens have been read
  pub(super) fn is_empty(&self) -> bool {
    self.peek().is_none()
  }

  /// Seeks past whitespace and comments
  pub(super) fn seek_whitespace_and_comments(&mut self) {
    while let Some(whitespace_or_comment!()) = self.peek_kind() {
      self.seek();
    };
  }

  /// Returns the amount of marks stored in the reader for asserting that we
  /// have cleaned them up properly.  This is a rather simple heuristic but
  /// it catches small mistakes
  pub(super) fn marks_len(&self) -> usize {
    self.marks.len()
  }

  /// Next Span in stream, or a Span representing the beginning of this empty
  /// file
  pub(super) fn next_span(&self) -> Span<W> {
    if let Some(peek) = self.peek() {
      // Either the next Span
      peek.span
    } else if let Some(last) = self.tokens.last() {
      // The last Span
      last.span
    } else {
      // Or a Span representing the beginning of an empty file, since that's
      // the only other possibility
      Span {
        start: 0,
        end: 0,
        handle: self.handle,
      }
    }
  }
}
