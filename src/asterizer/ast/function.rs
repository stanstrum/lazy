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

use super::FunctionDeclaration;

#[derive(Debug)]
pub(crate) struct Function {
  pub decl: FunctionDeclaration,
}

impl MakeAst for Function {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    stream.push_mark();

    let Some(decl) = FunctionDeclaration::make(stream)? else {
      stream.pop_mark();

      return Ok(None);
    };

    stream.drop_mark();

    Ok(Some(Self { decl }))
  }
}
