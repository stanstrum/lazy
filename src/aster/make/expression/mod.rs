/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::arch::x86_64::_MM_FROUND_TO_POS_INF;

use crate::aster::consts::operator::UNARY_PFX_MAP;

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

use enum_iterator::{Sequence, all};

#[derive(Debug, Sequence)]
enum PEMDAS {
  Dot,
  SubscriptCall,
  Unary,
  Bit,
  Exp,
  MulDivMod,
  AddSub,
  Comparison,
  Assignation,
  Pipe,
}

impl PEMDAS {
  fn includes(&self, op: &Operator) -> bool {
    match self {
      PEMDAS::Dot => {
        match op {
          Operator::Binary(BinaryOperator::Dot) => true,
          Operator::Binary(BinaryOperator::DerefDot) => true,
          _ => false
        }
      },
      PEMDAS::SubscriptCall => {
        match op {
          Operator::UnarySfx(UnarySfxOperator::Subscript { .. }) => true,
          Operator::UnarySfx(UnarySfxOperator::Call { .. }) => true,
          _ => false
        }
      },
      PEMDAS::Unary => {
        match op {
          Operator::UnarySfx(UnarySfxOperator::PostIncrement) => true,
          Operator::UnarySfx(UnarySfxOperator::PostDecrement) => true,
          Operator::UnaryPfx(UnaryPfxOperator::Ref) => true,
          Operator::UnaryPfx(UnaryPfxOperator::Deref) => true,
          Operator::UnaryPfx(UnaryPfxOperator::Not) => true,
          Operator::UnaryPfx(UnaryPfxOperator::Neg) => true,
          Operator::UnaryPfx(UnaryPfxOperator::NotNeg) => true,
          Operator::UnaryPfx(UnaryPfxOperator::PreIncrement) => true,
          Operator::UnaryPfx(UnaryPfxOperator::PreDecrement) => true,
          _ => false
        }
      },
      PEMDAS::Bit => {
        match op {
          Operator::Binary(BinaryOperator::BitAnd) => true,
          Operator::Binary(BinaryOperator::BitOr) => true,
          Operator::Binary(BinaryOperator::BitXOR) => true,
          _ => false
        }
      },
      PEMDAS::Exp => {
        match op {
          Operator::Binary(BinaryOperator::Exp) => true,
          _ => false
        }
      },
      PEMDAS::MulDivMod => {
        match op {
          Operator::Binary(BinaryOperator::Mul) => true,
          Operator::Binary(BinaryOperator::Div) => true,
          Operator::Binary(BinaryOperator::Mod) => true,
          _ => false
        }
      },
      PEMDAS::AddSub => {
        match op {
          Operator::Binary(BinaryOperator::Add) => true,
          Operator::Binary(BinaryOperator::Sub) => true,
          _ => false
        }
      },
      PEMDAS::Comparison => {
        match op {
          Operator::Binary(BinaryOperator::Equals) => true,
          Operator::Binary(BinaryOperator::NotEquals) => true,
          Operator::Binary(BinaryOperator::Greater) => true,
          Operator::Binary(BinaryOperator::GreaterThanEquals) => true,
          Operator::Binary(BinaryOperator::LessThan) => true,
          Operator::Binary(BinaryOperator::LessThanEquals) => true,
          _ => false
        }
      },
      PEMDAS::Assignation => {
        match op {
          Operator::Binary(BinaryOperator::AddAssign) => true,
          Operator::Binary(BinaryOperator::SubAssign) => true,
          Operator::Binary(BinaryOperator::MulAssign) => true,
          Operator::Binary(BinaryOperator::DivAssign) => true,
          Operator::Binary(BinaryOperator::ExpAssign) => true,
          Operator::Binary(BinaryOperator::ModAssign) => true,
          Operator::Binary(BinaryOperator::LogicalAndAssign) => true,
          Operator::Binary(BinaryOperator::LogicalOrAssign) => true,
          Operator::Binary(BinaryOperator::LogicalXORAssign) => true,
          Operator::Binary(BinaryOperator::BitAndAssign) => true,
          Operator::Binary(BinaryOperator::BitOrAssign) => true,
          Operator::Binary(BinaryOperator::BitXORAssign) => true,
          Operator::Binary(BinaryOperator::ArithmeticShrAssign) => true,
          Operator::Binary(BinaryOperator::LogicalShrAssign) => true,
          Operator::Binary(BinaryOperator::LogicalShlAssign) => true,
          Operator::Binary(BinaryOperator::AssignPipe) => true,
          Operator::Binary(BinaryOperator::Assign) => true,
          _ => false
        }
      },
      PEMDAS::Pipe => {
        match op {
          Operator::Binary(BinaryOperator::Pipe) => true,
          _ => false
        }
      },
    }
  }
}

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

  pub fn make_binary_half(reader: &mut SourceReader) -> AsterResult<(BinaryOperator, Expression)> {
    let start = reader.offset();

    let result = 'result: {
      seek::optional_whitespace(reader)?;

      let op = 'find_operator: {
        for (txt, variant) in consts::operator::BIN_MAP.into_iter() {
          if seek::begins_with(reader, txt) {
            break 'find_operator Some(variant.to_owned());
          };
        };

        None
      };

      if op.is_none() {
        break 'result None;
      };

      seek::optional_whitespace(reader)?;

      let Ok(expr) = Expression::make_expr_body(reader) else {
        break 'result None;
      };

      Some((op.unwrap(), expr))
    };

    if let Some(result) = result {
      Ok(result)
    } else {
      reader.to(start).unwrap();

      ExpectedSnafu {
        what: "Binary Operator Latter Half",
        offset: reader.offset()
      }.fail()
    }
  }

  pub fn make_unary_pfx(reader: &mut SourceReader) -> AsterResult<(UnaryPfxOperator)> {
    let start = reader.offset();

    let result = 'result: {
      for (txt, variant) in consts::operator::UNARY_PFX_MAP.into_iter() {
        if seek::begins_with(reader, txt) {
          break 'result Some(variant.to_owned());
        };
      };

      None
    };

    if let Some(result) = result {
      Ok(result)
    } else {
      reader.to(start).unwrap();

      ExpectedSnafu {
        what: "Unary Prefix Operator",
        offset: reader.offset()
      }.fail()
    }
  }

  pub fn make_unary_sfx(reader: &mut SourceReader) -> AsterResult<(UnarySfxOperator)> {
    let start = reader.offset();

    let result = 'result: {
      for (txt, variant) in consts::operator::UNARY_SFX_MAP.into_iter() {
        if seek::begins_with(reader, txt) {
          break 'result Some(variant.to_owned());
        };
      };

      None
    };

    if let Some(result) = result {
      Ok(result)
    } else {
      reader.to(start).unwrap();

      ExpectedSnafu {
        what: "Unary Prefix Operator",
        offset: reader.offset()
      }.fail()
    }
  }

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let mut exprs: Vec<Expression> = vec![Expression::make_expr_body(reader)?];
    let mut ops: Vec<Operator> = vec![];

    loop {
      if let Ok((op, expr)) = Expression::make_binary_half(reader) {
        exprs.push(expr);
        ops.push(Operator::Binary(op));
      } else if let Ok(op) = Expression::make_unary_sfx(reader) {
        ops.push(Operator::UnarySfx(op));
      } else if let Ok(op) = Expression::make_unary_pfx(reader) {
        ops.push(Operator::UnaryPfx(op));
      } else {
        break;
      }
    };

    while ops.len() >= 1 {
      for state in all::<PEMDAS>() {
        'pemdas: loop {
          for i in 0..ops.len() {
            let op = &ops[i];

            if state.includes(op) {
              match op {
                Operator::Binary(_) => {
                  let a = Box::new(exprs[i].to_owned());
                  let b = Box::new(exprs.remove(i + 1));

                  let Operator::Binary(op) = ops.remove(i) else {
                    unreachable!();
                  };

                  exprs[i] = Expression::BinaryOperator(BinaryOperatorExpressionAST {
                    a, b, op, out: Type::Unresolved
                  });

                  continue 'pemdas;
                },
                Operator::UnarySfx(UnarySfxOperator::Subscript { .. }) => {
                  todo!("unarysfxoperator subscript");
                },
                Operator::UnarySfx(UnarySfxOperator::Call { .. }) => {
                  todo!("unarysfxoperator call");
                },
                Operator::UnarySfx(_) => {
                  let expr = Box::new(exprs[i].to_owned());
                  let Operator::UnarySfx(op) = ops.remove(i) else {
                    unreachable!();
                  };

                  let new_expr = Expression::UnaryOperator(UnaryOperatorExpressionAST {
                    span: expr.span(),
                    out: Type::Unresolved,
                    expr, op: UnaryOperator::UnarySfx(op)
                  });

                  exprs[i] = new_expr;
                },
                _ => todo!("{:#?}", op)
              }
            };
          };

          break;
        }
      };
    };

    Ok(exprs.pop().unwrap())
  }
}
