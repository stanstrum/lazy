/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

 use super::super::{
  ast::*,
  SourceReader,
  AsterResult,
  errors::*
};

impl MemberFunctionDeclAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    NotImplementedSnafu {
      what: "Member Function Decl",
      offset: reader.offset()
    }.fail()
  }
}
