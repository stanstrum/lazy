/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::tokenizer::Token;

pub(crate) mod error;
pub(crate) use error::AsterizerError;
use error::*;

pub(crate) fn asterize(_toks: Vec<Token>) -> Result<(), AsterizerError> {
  NotImplementedSnafu { message: "asterize" }.fail()
}
