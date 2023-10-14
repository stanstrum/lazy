/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::intrinsics;

use super::*;

impl Checker {
  pub fn resolve_function(&mut self, func: &mut FunctionAST) -> TypeCheckResult<()> {
    let args = &mut func.decl.args;

    for ty in args.values_mut() {
      self.resolve_type(ty)?;
    };

    let block = &mut func.body;

    self.stack.push(ScopePointer::Block(block));

    if block.returns_last {
      self.resolve_block_expression(block, Some(&func.decl.ret.e))?;
    } else {
      self.resolve_block_expression(block, None)?;
    };

    self.stack.pop();

    if func.body.returns_last {
      let last_child = func.body.children.last().unwrap();

      match last_child {
        BlockExpressionChild::Binding(_) => todo!("throw error for returning value of binding"),
        BlockExpressionChild::Expression(expr) => {
          if !extends(&expr.type_of().unwrap(), &func.decl.ret.e) {
            return IncompatibleTypeSnafu {
              span: last_child.span(),
              what: "Return last expression",
              with: "return type",
            }.fail();
          };
        },
      };
    } else if !extends(&func.decl.ret.e, &Type::Intrinsic(intrinsics::VOID)) {
      todo!("throw error for no return last/void mismatch");
    };

    Ok(())
  }
}
