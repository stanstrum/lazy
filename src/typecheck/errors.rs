/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum TypeCheckError {
  #[snafu(display("Not implemented: {what}"))]
  NotImplemented { what: String },

  #[snafu(display("Unknown identifier: {text}"))]
  UnknownIdent { text: String }
}

pub type TypeCheckResult<T> = Result<T, TypeCheckError>;
