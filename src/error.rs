/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use snafu::prelude::*;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum CompilationError {
  #[snafu(display("{message}"))]
  Argument { message: String },

  #[snafu(display("{error}"))]
  InputFile { error: std::io::Error },

  #[snafu(display("{error}"))]
  Tokenization { error: crate::tokenizer::TokenizationError },

  #[snafu(display("{error}"))]
  Asterization { error: crate::asterizer::AsterizerError },
}
