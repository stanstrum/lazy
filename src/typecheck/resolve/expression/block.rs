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
  TypeOf,
  assignable
};

use crate::aster::{
  ast::*,
  intrinsics,
};

impl Checker {
  pub fn resolve_block_expression(&mut self, block: &mut BlockExpressionAST, coerce_to: Option<&Type>) -> TypeCheckResult<()> {
    let len = block.children.len();

    if !block.returns_last {
      block.out = Type::Intrinsic(intrinsics::VOID);
    };

    for (i, expr) in block.children.iter_mut().enumerate() {
      match expr {
        BlockExpressionChild::Binding(binding) => {
          if binding.ty.is_some() {
            self.resolve_type(binding.ty.as_mut().unwrap())?;
          };

          if binding.value.is_some() {
            self.resolve_expression(
              binding.value.as_mut().unwrap(),
              binding.ty.as_ref().map(|ast| &ast.e)
            )?;
          };

          if binding.ty.is_none() {
            let value = binding.value.as_ref().unwrap();

            let value_ty = value.as_ref().type_of_expect(value.span())?;

            binding.ty = Some(TypeAST {
              span: value.span(),
              e: value_ty
            });
          };

          if let Some(value) = &binding.value {
            if !assignable(&(value.type_of_expect_implicit()?), &binding.ty.as_ref().unwrap().e) {
              return IncompatibleTypeSnafu {
                span: value.span(),
                what: "Value expression",
                with: "binding type",
              }.fail();
            };
          };

          block.vars.insert(binding.ident.clone(), binding);
        },
        BlockExpressionChild::Expression(expr) => {
          if i + 1 == len && block.returns_last {
            self.resolve_expression(expr, coerce_to)?;

            block.out = expr.type_of()
              .expect("resolve expression did resolve out type");
          } else {
            self.resolve_expression(expr, None)?;
          };
        },
      };
    };

    Ok(())
  }
}
