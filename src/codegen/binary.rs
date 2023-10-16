/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::values::{AnyValueEnum, BasicValueEnum};

use super::{
  Codegen,
  CodeGenResult
};
use crate::aster::ast::*;

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  pub fn generate_binary_operator(&mut self, binary: &BinaryOperatorExpressionAST) -> CodeGenResult<Option<AnyValueEnum<'ctx>>> {
    match &binary.op {
      BinaryOperator::Assign => {
        let dest = self.generate_expr(&binary.a)?
          .expect("generate_expr dest doesn't return for assign")
          .into_pointer_value();

        let value = self.generate_expr(&binary.b)?
          .expect("generate_expr value doesn't return for assign");

        // dbg!(&dest);
        // dbg!(&value);

        self.builder.build_store::<BasicValueEnum>(dest, value.try_into().unwrap());

        Ok(None)
      },
      BinaryOperator::Add => {
        let a = self.generate_expr(&binary.a)?
          .expect("generate_expr didn't return for add")
          .into_int_value();

        let b = self.generate_expr(&binary.b)?
          .expect("generate_expr didn't return for add")
          .into_int_value();

        let value = self.builder.build_int_add(a, b, "add").into();

        Ok(Some(value))
      }
      _ => todo!("generate_binary_operator {:?}", &binary.op)
    }
  }
}
