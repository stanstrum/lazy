/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum AsterError {
  #[snafu(display("Expected {what}"))]
  Expected { what: String, offset: usize },

  #[snafu(display("Unknown {what}"))]
  Unknown { what: String, offset: usize },

  #[snafu(display("NotImplemented {what}"))]
  NotImplemented { what: String, offset: usize },
}

pub type AsterResult<T> = Result<T, AsterError>;
