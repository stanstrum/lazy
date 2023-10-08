/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::{
  ast::*,
  consts
};

use crate::colors::*;

impl std::string::ToString for BinaryOperator {
  fn to_string(&self) -> String {
    consts::operator::BIN_MAP
      .into_iter()
      .find_map(
        |(key, val)|
          if val == self {
            Some(key)
          } else {
            None
          }
      ).unwrap_or_else(|| panic!("no operator for variant {:#?}", self)).to_string()
  }
}

impl std::string::ToString for UnaryPfxOperator {
  fn to_string(&self) -> String {
    consts::operator::UNARY_PFX_MAP
      .into_iter()
      .find_map(
        |(key, val)|
          if val == self {
            Some(key)
          } else {
            None
          }
      ).unwrap_or_else(|| panic!("no operator for variant {:#?}", self)).to_string()
  }
}

impl std::string::ToString for UnarySfxOperator {
  fn to_string(&self) -> String {
    consts::operator::UNARY_SFX_MAP
      .into_iter()
      .find_map(
        |(key, val)|
          if val == self {
            Some(key)
          } else {
            None
          }
      ).unwrap_or_else(|| panic!("no operator for variant {:#?}", self)).to_string()
  }
}

impl std::string::ToString for UnaryOperator {
  fn to_string(&self) -> String {
    match self {
      UnaryOperator::UnaryPfx(pfx) => pfx.to_string(),
      UnaryOperator::UnarySfx(sfx) => sfx.to_string(),
    }
  }
}

impl std::string::ToString for Operator {
  fn to_string(&self) -> String {
    match self {
      Operator::UnaryPfx(op) => op.to_string(),
      Operator::UnarySfx(op) => op.to_string(),
      Operator::Binary(op) => op.to_string(),
    }
  }
}

impl std::string::ToString for Expression {
  fn to_string(&self) -> String {
    match self {
      Expression::Atom(a) => a.to_string(),
      Expression::Block(a) => a.to_string(),
      Expression::SubExpression(a) => a.to_string(),
      Expression::ControlFlow(a) => a.to_string(),
      Expression::BinaryOperator(BinaryOperatorExpressionAST { a, b, op, .. }) => {
        match op {
          BinaryOperator::Dot | BinaryOperator::DerefDot =>
            format!(
              "{}{TEAL}{}{CLEAR}{}",
              a.to_string(),
              op.to_string(),
              b.to_string()
            ),
          _ => format!(
            "{} {TEAL}{}{CLEAR} {}",
            a.to_string(),
            op.to_string(),
            b.to_string()
          )
        }
      },
      Expression::UnaryOperator(UnaryOperatorExpressionAST { expr, op, ..}) => {
        match op {
          UnaryOperator::UnarySfx(UnarySfxOperator::Subscript { arg }) => {
            format!("{}[{}]", expr.to_string(), arg.to_string())
          },
          UnaryOperator::UnarySfx(UnarySfxOperator::Call { args }) => {
            format!(
              "{}({})",
              expr.to_string(),
              args.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", ")
            )
          },
          UnaryOperator::UnarySfx(_) => {
            format!("{}{TEAL}{}", expr.to_string(), op.to_string())
          },
          UnaryOperator::UnaryPfx(UnaryPfxOperator::MutRef) => {
            format!("{TEAL}{}{CLEAR} {}", op.to_string(), expr.to_string())
          },
          UnaryOperator::UnaryPfx(_) => {
            format!("{TEAL}{}{CLEAR}{}", op.to_string(), expr.to_string())
          },
        }
      },
    }
  }
}
