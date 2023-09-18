/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum CodeGenError {
  #[snafu(display("Not implemented: {what}"))]
  NotImplemented { what: String }
}

pub type CodeGenResult<T> = Result<T, CodeGenError>;
