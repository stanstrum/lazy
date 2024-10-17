use crate::compiler::Result;
use crate::tokenizer::{
  PeekReader,
  Tokenizer,
  TokenKind,
  NumericKind,
  error::*
};

#[derive(Debug)]
pub(crate) enum NumericState {
  Binary,
  Octal,
  Decimal,
  Hexadecimal,
}

impl Tokenizer {
  pub(in crate::tokenizer) fn numeric(&mut self, reader: &mut PeekReader) -> Result {
    let mut content = String::new();
    let mut state = None;

    let start = reader.span_start();

    loop {
      let Some(item) = reader.next() else {
        break;
      };

      let item = item?;
      match (&state, item.ch) {
        | (Some(NumericState::Binary), binary!())
        | (Some(NumericState::Octal), octal!())
        | (Some(NumericState::Decimal), decimal!())
        | (Some(NumericState::Hexadecimal), hexademical!())
        | (Some(_), '_')
        | (None, '0') => {},
        (None, decimal!()) => {
          state = Some(NumericState::Decimal);
        },
        (None, 'b' | 'o' | 'd' | 'x') if content == "0" => {
          state = Some({
            match item.ch {
              'b' => NumericState::Binary,
              'o' => NumericState::Octal,
              'd' => NumericState::Decimal,
              'x' => NumericState::Hexadecimal,
              _ => unreachable!(),
            }
          });

          content.clear();
          continue;
        },
        _ => break,
      };

      content.push(item.ch);
    };

    if content.is_empty() {
      return ExpectedSnafu { what: What::Numeric }.fail()?;
    };

    let state = state.unwrap_or(NumericState::Decimal);
    let kind = NumericKind::from_state_and_content(state, &content)?;

    self.push_tok(TokenKind::Numeric(kind), start, reader.position);

    Ok(())
  }
}
