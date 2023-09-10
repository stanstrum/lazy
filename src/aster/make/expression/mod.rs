/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::{
  super::{
    SourceReader,
    AsterResult,
    ast::*,
    errors::*
  },
  try_make
};

mod atom;
mod block;
mod sub;
mod control_flow;

impl Expression {
  fn make_expr_body(reader: &mut SourceReader) -> AsterResult<Self> {
    if let Some(ctrl_flow) = try_make!(ControlFlowAST::make, reader) {
      Ok(Expression::ControlFlow(ctrl_flow))
    } else if let Some(expr) = try_make!(BlockExpressionAST::make, reader) {
      Ok(Expression::Block(expr))
    } else if let Some(expr) = try_make!(AtomExpressionAST::make, reader) {
      Ok(Expression::Atom(expr))
    } else if let Some(sub_expr) = try_make!(SubExpressionAST::make, reader) {
      Ok(Expression::SubExpression(sub_expr))
    } else {
      ExpectedSnafu {
        what: "Expression (BlockExpression, AtomExpression)",
        offset: reader.offset()
      }.fail()
    }
  }

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    Expression::make_expr_body(reader)
  }
}
