use snafu::prelude::*;

use crate::tokenizer::TokenizationError;
use crate::asterizer::AsterizerError;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum CompilationError {
  #[snafu(display("{message}"))]
  Argument { message: String },

  #[snafu(display("{error}"))]
  InputFile { error: std::io::Error },

  #[snafu(display("{error}"))]
  Tokenization { error: TokenizationError },

  #[snafu(display("{error}"))]
  Asterization { error: AsterizerError },
}
