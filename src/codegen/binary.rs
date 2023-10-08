/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::values::AnyValueEnum;

use super::{
  Codegen,
  CodeGenResult
};
use crate::aster::ast::*;

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  pub fn generate_binary_operator(&mut self, binary: &BinaryOperatorExpressionAST) -> CodeGenResult<Option<AnyValueEnum<'ctx>>> {
    match &binary.op {
      BinaryOperator::Assign => {
        let dest = self.generate_expr(&binary.a)?;

        let value = self.generate_expr(&binary.b)?;

        // self.builder.build_store(, value)

        todo!()
      },
      _ => todo!("generate_binary_operator {:?}", &binary.op)
    }
  }
}
