/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::asterizer::{
  error::ExpectedSnafu, AsterizerError, MakeAst, TokenStream
};

use super::{BlockExpression, FunctionDeclaration};

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Function {
  pub decl: FunctionDeclaration,
  body: BlockExpression
}

impl MakeAst for Function {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(decl) = stream.make::<FunctionDeclaration>()? else {
      return Ok(None);
    };

    let Some(body) = stream.make::<BlockExpression>()? else {
      return ExpectedSnafu {
        what: "a block expression"
      }.fail();
    };

    Ok(Some(Self { decl, body }))
  }
}
