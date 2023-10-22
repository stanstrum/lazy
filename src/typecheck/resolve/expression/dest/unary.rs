/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

 use crate::typecheck::{
  Checker,
  TypeCheckResult,
  errors::*,
  type_of::{
    dereference_type,
    is_array,
    get_element_of
  }
};

use crate::aster::{
  ast::*,
  intrinsics
};

impl Checker {
  pub fn resolve_dest_unary_operator(&mut self, unary: &mut UnaryOperatorExpressionAST) -> TypeCheckResult<Type> {
    match &mut unary.op {
      UnaryOperator::UnarySfx(UnarySfxOperator::Subscript { arg, dest }) => {
        let span = unary.expr.span();

        let ptr_arr_ty = self.resolve_expression(&mut unary.expr, None)?;
        self.resolve_expression(arg, Some(&Type::Intrinsic(intrinsics::USIZE)))?;

        let arr_ty = dereference_type(&ptr_arr_ty, span.clone())?;
        if !is_array(&arr_ty) {
          return IncompatibleTypeSnafu {
            span,
            what: "Expression",
            with: "array index",
          }.fail();
        };

        *dest = true;

        let out = get_element_of(&arr_ty, span)?;
        unary.out = out;

        Ok(unary.out.clone())
      },
      _ => todo!("resolve_dest_unary_operator {:#?}", &unary.op)
    }
  }
}
