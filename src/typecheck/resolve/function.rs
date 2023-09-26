/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::*;

impl Checker {
  pub fn resolve_function(&mut self, func: &mut FunctionAST) -> TypeCheckResult<()> {
    let args = &mut func.decl.args;

    for (ident, ty) in args.iter_mut() {
      self.resolve_type(ty)?;
    };

    let block = &mut func.body;

    self.stack.push(ScopePointer::Block(block));
    self.resolve_block_expression(block)?;
    self.stack.pop();

    Ok(())
  }
}
