/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::{
  values::{
    BasicMetadataValueEnum,
    AnyValue,
    AnyValueEnum,
    FunctionValue,
    BasicValueEnum
  },
  types::BasicTypeEnum
};

use super::{
  Codegen,
  CodeGenResult
};

use crate::aster::ast::*;

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  pub fn generate_unary_operator(&mut self, unary: &UnaryOperatorExpressionAST) -> CodeGenResult<Option<AnyValueEnum<'ctx>>> {
    Ok(match &unary.op {
      UnaryOperator::UnarySfx(UnarySfxOperator::Call { args }) => {
        let callee = self.generate_expr(&unary.expr, None)?
          .expect("could not generate callee value")
          .into_function_value();

        let mut args = args.iter()
          .map(|arg| self.generate_expr(arg, None))
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

        for (i, (arg, dest_ty)) in
          args.iter_mut().zip(
            callee
              .get_param_iter()
              .map(|ty| ty.get_type()
            )
          ).enumerate()
        {
          let casted_arg = self.builder.build_bitcast::<BasicTypeEnum, BasicValueEnum>(
            (*arg).try_into().unwrap(),
            dest_ty,
            format!("bitcast arg({i})").as_str()
          );

          *arg = BasicMetadataValueEnum::from(casted_arg);
        };

        let callsite = self.builder.build_call::<FunctionValue<'ctx>>(
          callee,
          args.as_slice(),
          "fncall"
        );

        Some(
          callsite.as_any_value_enum()
        )
      },
      UnaryOperator::UnarySfx(UnarySfxOperator::Cast { to, method }) => {
        let to_ty = self.generate_type(&to.e)?
          .to_basic_metadata();

        let value = self.generate_expr(unary.expr.as_ref(), None)?
            .expect("generate_expr returned None for cast")
            .into_int_value();

        match method {
          Some(CastMethod::Truncate) => {
            let casted = self.builder.build_int_cast(value, to_ty.into_int_type(), "cast_int_truncate");

            Some(casted.as_any_value_enum())
          },
          Some(CastMethod::ZeroExtend) => {
            let casted = self.builder.build_int_z_extend_or_bit_cast(
              value,
              to_ty.into_int_type(),
              "cast_int_zero_extend"
            );

            Some(casted.as_any_value_enum())
          }
          None => {
            Some(value.as_any_value_enum())
          },
          #[allow(unused)]
          _ => todo!("cast method {method:?}")
        }
      },
      UnaryOperator::UnarySfx(UnarySfxOperator::Subscript { arg, dest }) => {
        let ptr = self.generate_expr(&unary.expr, None)?
          .expect("generate_expr didn't return for subscript dest")
          .into_pointer_value();

        let offset = self.generate_expr(arg, None)?
          .expect("generate_expr didn't return for subscript value")
          .into_int_value();

        let nth_ptr = unsafe {
          self.builder.build_gep(ptr, &[offset], "get_nth_ptr")
        };

        Some({
          if *dest {
            nth_ptr.as_any_value_enum()
          } else {
            self.builder.build_load(nth_ptr, "get_nth_ptr_value")
              .as_any_value_enum()
          }
        })
      },
      UnaryOperator::UnaryPfx(_) => todo!("generate_unary_operator {:#?}", &unary.op),
      UnaryOperator::UnarySfx(_) => todo!("generate_unary_operator {:#?}", &unary.op),
    })
  }
}
