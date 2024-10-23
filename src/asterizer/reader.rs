use crate::{tokenizer::{
  SpanStart,
  Token,
  TokenKind,
}, whitespace_or_comment};

/// A reader for Tokens that allows for peeking, reading, setting, and resetting
/// the internal reader position at will
pub(super) struct TokenReader {
  /// The tokens taken from the tokenization stage
  tokens: Vec<Token>,
  /// The saved position markers that can be used to restore the internal reader
  /// position
  marks: Vec<usize>,
  /// The current position of the reader along the Token vector
  position: usize,
}

impl TokenReader {
  /// Creates a TokenReader provided Tokens
  pub(super) fn new(tokens: Vec<Token>) -> Self {
    Self {
      tokens,
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
  pub(super) fn peek(&self) -> Option<&Token> {
    self.tokens.get(self.position)
  }

  /// Peeks the next TokenKind without advancing the reader position
  #[allow(unused)]
  pub(super) fn peek_kind(&self) -> Option<&TokenKind> {
    self.peek().map(|tok| &tok.kind)
  }

  /// Reads the next Token and advances the reader position
  pub(super) fn next(&mut self) -> Option<&Token> {
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
  pub(super) fn get_start(&self) -> SpanStart {
    SpanStart(self.get_position())
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
}
