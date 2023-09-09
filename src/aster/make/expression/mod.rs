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
    errors::*,
    seek,
    consts
  },
  try_make
};

mod atom;
mod block;
mod sub;
mod control_flow;

impl Expression {
  fn make_body(reader: &mut SourceReader) -> AsterResult<Self> {
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
    let start = reader.offset();

    let first =  Expression::make_body(reader)?;

    let mut exprs: Vec<Expression> = vec![first];
    let mut ops: Vec<BinaryOperator> = vec![];

    loop {
      let start_of_nth = reader.offset();

      seek::optional_whitespace(reader)?;

      let op = if seek::begins_with(reader, consts::operator::ASSIGN) {
        BinaryOperator::Assign
      } else if seek::begins_with(reader, consts::operator::ADD_ASSIGN) {
        BinaryOperator::AddAssign
      } else if seek::begins_with(reader, consts::operator::SUB_ASSIGN) {
        BinaryOperator::SubAssign
      } else if seek::begins_with(reader, consts::operator::MUL_ASSIGN) {
        BinaryOperator::MulAssign
      } else if seek::begins_with(reader, consts::operator::DIV_ASSIGN) {
        BinaryOperator::DivAssign
      } else if seek::begins_with(reader, consts::operator::EXP_ASSIGN) {
        BinaryOperator::ExpAssign
      } else if seek::begins_with(reader, consts::operator::MOD_ASSIGN) {
        BinaryOperator::ModAssign
      } else if seek::begins_with(reader, consts::operator::LOGICALAND_ASSIGN) {
        BinaryOperator::LogicalAndAssign
      } else if seek::begins_with(reader, consts::operator::LOGICALOR_ASSIGN) {
        BinaryOperator::LogicalOrAssign
      } else if seek::begins_with(reader, consts::operator::LOGICALXOR_ASSIGN) {
        BinaryOperator::LogicalXORAssign
      } else if seek::begins_with(reader, consts::operator::BITAND_ASSIGN) {
        BinaryOperator::BitAndAssign
      } else if seek::begins_with(reader, consts::operator::BITOR_ASSIGN) {
        BinaryOperator::BitOrAssign
      } else if seek::begins_with(reader, consts::operator::BITXOR_ASSIGN) {
        BinaryOperator::BitXORAssign
      } else if seek::begins_with(reader, consts::operator::ASHR_ASSIGN) {
        BinaryOperator::ArithmeticShrAssign
      } else if seek::begins_with(reader, consts::operator::LSHR_ASSIGN) {
        BinaryOperator::LogicalShrAssign
      } else if seek::begins_with(reader, consts::operator::LSHL_ASSIGN) {
        BinaryOperator::LogicalShlAssign
      } else if seek::begins_with(reader, consts::operator::PIPE_ASSIGN) {
        BinaryOperator::AssignPipe
      } else if seek::begins_with(reader, consts::operator::DOT) {
        BinaryOperator::Dot
      } else if seek::begins_with(reader, consts::operator::DEREF_DOT) {
        BinaryOperator::DerefDot
      } else if seek::begins_with(reader, consts::operator::ADD) {
        BinaryOperator::Add
      } else if seek::begins_with(reader, consts::operator::SUB) {
        BinaryOperator::Sub
      } else if seek::begins_with(reader, consts::operator::EXP) {
        BinaryOperator::Exp
      } else if seek::begins_with(reader, consts::operator::MUL) {
        BinaryOperator::Mul
      } else if seek::begins_with(reader, consts::operator::DIV) {
        BinaryOperator::Div
      } else if seek::begins_with(reader, consts::operator::MOD) {
        BinaryOperator::Mod
      } else if seek::begins_with(reader, consts::operator::EQUALS) {
        BinaryOperator::Equals
      } else if seek::begins_with(reader, consts::operator::NOTEQUALS) {
        BinaryOperator::NotEquals
      } else if seek::begins_with(reader, consts::operator::GT) {
        BinaryOperator::Greater
      } else if seek::begins_with(reader, consts::operator::GEQ) {
        BinaryOperator::GreaterThanEquals
      } else if seek::begins_with(reader, consts::operator::LT) {
        BinaryOperator::LessThan
      } else if seek::begins_with(reader, consts::operator::LEQ) {
        BinaryOperator::LessThanEquals
      } else if seek::begins_with(reader, consts::operator::LOGICALAND) {
        BinaryOperator::LogicalAnd
      } else if seek::begins_with(reader, consts::operator::LOGICALOR) {
        BinaryOperator::LogicalOr
      } else if seek::begins_with(reader, consts::operator::LOGICALXOR) {
        BinaryOperator::LogicalXOR
      } else if seek::begins_with(reader, consts::operator::BITAND) {
        BinaryOperator::BitAnd
      } else if seek::begins_with(reader, consts::operator::BITOR) {
        BinaryOperator::BitOr
      } else if seek::begins_with(reader, consts::operator::BITXOR) {
        BinaryOperator::BitXOR
      } else if seek::begins_with(reader, consts::operator::ASHR) {
        BinaryOperator::ArithmeticShr
      } else if seek::begins_with(reader, consts::operator::LSHR) {
        BinaryOperator::LogicalShr
      } else if seek::begins_with(reader, consts::operator::LSHL) {
        BinaryOperator::LogicalShl
      } else if seek::begins_with(reader, consts::operator::PIPE) {
        BinaryOperator::Pipe
      } else {
        reader.to(start_of_nth).unwrap();

        break;
      };

      seek::optional_whitespace(reader)?;

      let Ok(nth) = Expression::make_body(reader) else {
        reader.to(start_of_nth).unwrap();

        break;
      };

      exprs.push(nth);
      ops.push(op);
    };

    if exprs.len() == 1 {
      Ok(exprs.pop().unwrap())
    } else {
      Ok(Self::Operator(OperatorExpressionAST {
        span: reader.span_since(start),
        out: Type::Unresolved,
        exprs, ops
      }))
    }
  }
}
