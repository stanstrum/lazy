/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod block;
pub(crate) use block::*;

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError
};

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum Expression {
  BlockExpression(BlockExpression)
}

impl MakeAst for Expression {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(block) = stream.make::<BlockExpression>()? {
        Some(Expression::BlockExpression(block))
      } else {
        None
      }
    })
  }
}
