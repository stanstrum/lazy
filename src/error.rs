/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

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
