/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod errors;
use errors::*;

use crate::aster::ast::NamespaceAST;

pub fn gen(_global: &NamespaceAST) -> CodeGenResult<(/* todo */)> {
  NotImplementedSnafu { what: "Code generation" }.fail()
}
