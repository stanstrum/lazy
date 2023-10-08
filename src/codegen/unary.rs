/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::values::{BasicMetadataValueEnum, AnyValue, AnyValueEnum, FunctionValue};

use super::{
  Codegen,
  CodeGenResult
};

use crate::aster::ast::*;

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  pub fn generate_unary_operator(&mut self, unary: &UnaryOperatorExpressionAST) -> CodeGenResult<Option<AnyValueEnum<'ctx>>> {
    Ok(match &unary.op {
      UnaryOperator::UnarySfx(UnarySfxOperator::Call { args }) => {
        let callee = self.generate_expr(&unary.expr)?
          .expect("could not generate callee value")
          .into_function_value();

        let args = args.iter()
          .map(|arg| self.generate_expr(arg))
          .collect::<Result<Vec<_>, _>>()?
          .iter()
          .map(
            |arg|
              arg.expect("could not generate argument value for fn call")
          )
          .map(
            |arg| BasicMetadataValueEnum::try_from(arg).unwrap()
          )
          .collect::<Vec<_>>();

        let name = self.unique_name("fncall");

        let callsite = self.builder.build_call::<FunctionValue<'ctx>>(
          callee,
          args.as_slice(),
          &name
        );

        Some(
          callsite.as_any_value_enum()
        )
      },
      UnaryOperator::UnaryPfx(_) => todo!("generate_unary_operator {:#?}", &unary.op),
      UnaryOperator::UnarySfx(_) => todo!("generate_unary_operator {:#?}", &unary.op),
    })
  }
}
