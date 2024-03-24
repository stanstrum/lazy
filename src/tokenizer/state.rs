#[derive(Debug)]
pub(super) enum NumericType {
  Binary,
  Octal,
  Decimal,
  Hexadecimal,
  FloatingPoint, // decimal only
}

#[derive(Debug)]
pub(super) enum StringType {
  Unicode, // 32bit unicode, length-prefixed -- default type
  C, // 7-bit ASCII, null-terminated (c"hello, world")
  Bytes, // 7-bit ASCII, not null-terminated (b"hello, world")
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
  StringLiteral {
    start: usize,
    escape_next: bool,
    ty: StringType,
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
