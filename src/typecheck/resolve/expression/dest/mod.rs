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
  pub fn resolve_dest_expression(&mut self, expr: &mut Expression) -> TypeCheckResult<Type> {
    match expr {
      Expression::Atom(atom) => self.resolve_dest_atom(atom),
      Expression::Block(_) => todo!("resolve_dest_expression block"),
      Expression::SubExpression(_) => todo!("resolve_dest_expression subexpression"),
      Expression::ControlFlow(_) => todo!("resolve_dest_expression controlflow"),
      Expression::UnaryOperator(unary) => self.resolve_dest_unary_operator(unary),
      Expression::BinaryOperator(binary) => self.resolve_dest_binary_operator(binary)
    }
  }
}
