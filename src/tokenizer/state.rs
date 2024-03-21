#[derive(Debug)]
pub(super) enum NumericType {
  Binary,
  Octal,
  Decimal,
  Hexadecimal,
  FloatingPoint, // decimal only
}

#[derive(Debug)]
pub(super) enum State {
  Base,
  CommentBegin {
    start: usize,
  },
  MultilineComment {
    start: usize,
    content: String,
  },
  LineComment {
    start: usize,
    content: String,
  },
  MultilineCommentEnding {
    start: usize,
    content: String
  },
  Text {
    start: usize,
    content: String
  },
  NumericLiteral {
    start: usize,
    ty: NumericType,
    content: String
  },
  Operator {
    start: usize,
    content: String,
  },
  Whitespace {
    start: usize,
    content: String
  },
}
