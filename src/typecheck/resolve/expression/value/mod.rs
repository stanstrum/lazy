/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod atom;
mod unary;
mod binary;

use crate::typecheck::{
  Checker,
  TypeCheckResult
};

use crate::aster::ast::*;

impl Checker {
  pub fn resolve_expression(&mut self, expr: &mut Expression, coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
    match expr {
      Expression::Atom(atom) => self.resolve_atom(atom, coerce_to),
      Expression::Block(_) => todo!("resolve block"),
      Expression::SubExpression(subexpr) => {
        subexpr.out = self.resolve_expression(&mut subexpr.e, coerce_to)?;

        Ok(subexpr.out.clone())
      },
      Expression::ControlFlow(flow) => self.resolve_control_flow(flow, coerce_to),
      Expression::BinaryOperator(binary) => self.resolve_binary_operator(binary, coerce_to),
      Expression::UnaryOperator(unary) => self.resolve_unary_operator(unary, coerce_to),
    }
  }
}
