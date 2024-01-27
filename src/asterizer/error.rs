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
pub(crate) enum AsterizerError {
  #[snafu(display("Not implemented: {message}"))]
  NotImplemented { message: String },

  #[snafu(display("Expected {what}"))]
  Expected { what: String }
}

impl From<AsterizerError> for CompilationError {
  fn from(error: AsterizerError) -> Self {
    Self::Asterization { error }
  }
}
