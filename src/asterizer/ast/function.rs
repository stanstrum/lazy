/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::asterizer::{
  TokenStream,
  AsterizerError,
  MakeAst
};

#[derive(Debug)]
pub(crate) struct Function {
  pub name: String,
}

impl MakeAst for Function {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    todo!()
  }
}
