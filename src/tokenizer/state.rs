#[allow(unused)]
#[derive(Debug)]
pub(super) enum NumericType {
  Binary,
  Octal,
  Decimal,
  Hexadecimal,
  FloatingPoint, // decimal only
}

#[derive(Debug, Clone, Copy)]
pub(super) enum StringType {
  Unicode, // 32bit unicode, length-prefixed -- default type
  C, // 7-bit ASCII, null-terminated (c"hello, world")
  Bytes, // 7-bit ASCII/raw bytes, not null-terminated (b"hello, world")
}

#[derive(Debug, Clone, Copy)]
pub(super) enum CharType {
  Unicode, // 32bit unicode -- default type
  Byte, // 7-bit ASCII (b' ', b'~')
}

#[derive(Debug, Clone, Copy)]
pub(super) enum StringEscapeReturnTo {
  String { ty: StringType },
  Char { ty: CharType },
}

#[derive(Debug)]
pub(super) enum StringEscapeType {
  Unicode { codepoint: String },
  Hexadecimal { codepoint: String },
  Octal { codepoint: String },
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
    content: String,
  },
  Text {
    start: usize,
    content: String,
  },
  NumericLiteral {
    start: usize,
    ty: NumericType,
    content: String,
  },
  StringLiteral {
    start: usize,
    ty: StringType,
    content: String,
  },
  CharLiteral {
    start: usize,
    ty: CharType,
    content: String,
  },
  StringEscape {
    start: usize,
    return_to: StringEscapeReturnTo,
    content: String,
    ty: Option<StringEscapeType>,
  },
  StringEscapeFinalize {
    start: usize,
    return_to: StringEscapeReturnTo,
    content: String,
    ty: Option<StringEscapeType>,
  },
  Operator {
    start: usize,
    content: String,
  },
  Whitespace {
    start: usize,
    content: String,
  },
  Invalid {
    start: usize,
    content: String,
  },
}
