/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use snafu::prelude::*;

use crate::CompilationError;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum TokenizationError {
  IOError { error: utf8_read::Error }
}

impl From<utf8_read::Error> for TokenizationError {
  fn from(error: utf8_read::Error) -> Self {
    Self::IOError { error }
  }
}

impl From<TokenizationError> for CompilationError {
  fn from(error: TokenizationError) -> Self {
    Self::Tokenization { error }
  }
}
