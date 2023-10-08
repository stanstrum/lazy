/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::values::{BasicValueEnum, BasicMetadataValueEnum, CallableValue, AnyValue};

use super::{
  Codegen,
  CodeGenResult
};

use crate::aster::ast::*;

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  pub fn generate_unary_operator(&mut self, unary: &UnaryOperatorExpressionAST) -> CodeGenResult<Option<BasicValueEnum<'ctx>>> {
    Ok(match &unary.op {
      UnaryOperator::UnarySfx(UnarySfxOperator::Call { args }) => {
        let callee = self.generate_expr(&unary.expr)?
          .expect("could not generate callee value")
          .into_pointer_value();

        let args = args.iter()
          .map(|arg| self.generate_expr(arg))
          .collect::<Result<Vec<_>, _>>()?
          .iter()
          .map(
            |arg|
              arg.expect("could not generate argument value for fn call")
          )
          .map(
            |arg| BasicMetadataValueEnum::from(arg)
          )
          .collect::<Vec<_>>();

        let name = self.unique_name("fncall");

        let callsite = self.builder.build_call(
          CallableValue::try_from(callee).unwrap(),
          args.as_slice(),
          &name
        );

        Some(
          callsite
            .try_as_basic_value()
            .unwrap_left()
        )
      },
      UnaryOperator::UnaryPfx(_) => todo!("generate_unary_operator {:#?}", &unary.op),
      UnaryOperator::UnarySfx(_) => todo!("generate_unary_operator {:#?}", &unary.op),
    })
  }
}
