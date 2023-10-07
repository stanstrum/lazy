/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::*;
use crate::make_get_span;

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
  Dot,
  DerefDot,

  Add,
  Sub,
  Mul,
  Div,
  Exp,
  Mod,

  Equals,
  NotEquals,
  Greater,
  GreaterThanEquals,
  LessThan,
  LessThanEquals,

  LogicalAnd,
  LogicalOr,
  LogicalXOR,
  BitAnd,
  BitOr,
  BitXOR,
  ArithmeticShr,
  LogicalShr,
  LogicalShl,

  // :)
  Pipe,

  Assign,

  AddAssign,
  SubAssign,
  MulAssign,
  DivAssign,
  ExpAssign,
  ModAssign,

  LogicalAndAssign,
  LogicalOrAssign,
  LogicalXORAssign,
  BitAndAssign,
  BitOrAssign,
  BitXORAssign,
  ArithmeticShrAssign,
  LogicalShrAssign,
  LogicalShlAssign,

  AssignPipe,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryPfxOperator {
  MutRef,
  Ref,
  Deref,
  Not,
  Neg,
  NotNeg,
  BitInvert,
  PreIncrement,
  PreDecrement,
}

#[derive(Debug, Clone)]
pub enum UnarySfxOperator {
  PostIncrement,
  PostDecrement,
  Subscript { arg: Box<Expression> },
  Call { args: Vec<Expression> }
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
  UnaryPfx(UnaryPfxOperator),
  UnarySfx(UnarySfxOperator),
}

#[derive(Debug)]
pub enum Operator {
  UnaryPfx(UnaryPfxOperator),
  UnarySfx(UnarySfxOperator),
  Binary(BinaryOperator),
}

impl GetSpan for BinaryOperatorExpressionAST {
  fn span(&self) -> Span {
    Span {
      start: std::cmp::min(
        self.a.span().start,
        self.b.span().start,
      ),
      end: std::cmp::max(
        self.a.span().end,
        self.b.span().end,
      ),
    }
  }
}

impl PartialEq for UnarySfxOperator {
  fn eq(&self, other: &Self) -> bool {
    std::mem::discriminant(self) == std::mem::discriminant(other)
  }
}

// Last words: "I know what I'm doing."
unsafe impl Sync for UnarySfxOperator {}
#[derive(Debug, Clone)]
pub struct UnaryOperatorExpressionAST {
  pub span: Span,
  pub out: Type,

  pub expr: Box<Expression>,
  pub op: UnaryOperator
}

#[derive(Debug, Clone)]
pub struct BinaryOperatorExpressionAST {
  pub out: Type,

  pub a: Box<Expression>,
  pub b: Box<Expression>,

  pub op: BinaryOperator
}

make_get_span![
  AtomExpressionAST,
  BlockExpressionAST
];
