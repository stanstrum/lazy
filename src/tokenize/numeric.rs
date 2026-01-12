use std::io::Read;

use super::{Tokenizer, Error};
use crate::tokenize::token::{NumericKind, NumericValue, Token};

impl<'pool, const N: usize, T: Read> Tokenizer<'pool, N, T> {
  pub(super) fn parse_and_push(&self, kind: Option<NumericKind>, content: &str) -> Result<Token, Error> {
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
      todo!("non-decimal floating-point: {kind:?}");
    };

    let value = if content.contains('.') {
      let Ok(value) = content.parse::<f64>() else {
        return Err(Error::InvalidNumeric);
      };

      NumericValue::F64(value)
    } else {
      let Ok(value) = u64::from_str_radix(content, radix) else {
        return Err(Error::InvalidNumeric);
      };

      NumericValue::U64(value)
    };

    Ok(Token::Numeric { kind, value })
  }
}
