use crate::tokenizer::{
  Token,
  SpanStart,
};

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

  /// Peeks the next token without advancing the reader position
  pub(super) fn peek(&self) -> Option<&Token> {
    self.tokens.get(self.position)
  }

  /// Reads the next token and advances the reader position
  pub(super) fn next(&mut self) -> Option<&Token> {
    let tok = self.tokens.get(self.position);
    self.position += 1;

    tok
  }

  /// Computes a SpanStart that refers to the next token to be read
  pub(super) fn get_start(&self) -> SpanStart {
    if let Some(peek) = self.peek() {
      // Either the next token, ...
      peek.span.into_start()
    } else if let Some(last) = self.tokens.last() {
      // Last token, ...
      last.span.into_start()
    } else {
      // Or a SpanStart representing the beginning of an empty file, since
      // there is no last Token
      SpanStart(0)
    }
  }
}
