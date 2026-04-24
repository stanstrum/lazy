use std::io::Read;

use lang::Compiler;

use super::{Tokenizer, Error};
use crate::token::{NumericKind, NumericValue, Token};
use lang::span::Span;

impl<'pool, C: Compiler, const N: usize, T: Read> Tokenizer<'pool, C, N, T> {
  pub(super) fn parse_and_push(&self, span: Span<C>, kind: Option<NumericKind>, content: &str) -> Result<Token, Error<C>> {
    let kind = kind.unwrap_or(NumericKind::Decimal);

    let radix = match kind {
      NumericKind::Binary => 2,
      NumericKind::Ternary => 3,
      NumericKind::Seximal => 6,
      NumericKind::Octal => 8,
      NumericKind::Decimal => 10,
      NumericKind::Hexadecimal => 16,
      NumericKind::Roman => unimplemented!("roman numerals"),
    };

    if content.contains('.') && !matches!(kind, NumericKind::Decimal) {
      return Err(Error::InvalidNumeric { span });
    };

    let parse = if content.contains('.') {
      content.parse().map(NumericValue::F64)
        .or(Err(Error::InvalidNumeric { span }))
    } else {
      u64::from_str_radix(content, radix).map(NumericValue::U64)
        .or(Err(Error::InvalidNumeric { span }))
    };

    parse.map(Token::Numeric)
  }
}
