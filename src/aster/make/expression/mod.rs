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

use enum_iterator::{Sequence, all};

#[derive(Debug, Sequence)]
enum PEMDAS {
  Dot,
  Bit,
  Exp,
  MulDivMod,
  AddSub,
  Comparison,
  Assignation,
  Pipe,
}

impl PEMDAS {
  fn includes(&self, op: &BinaryOperator) -> bool {
    match self {
      PEMDAS::Dot => {
        match op {
          BinaryOperator::Dot => true,
          BinaryOperator::DerefDot => true,
          _ => false
        }
      },
      PEMDAS::Bit => {
        match op {
          BinaryOperator::BitAnd => true,
          BinaryOperator::BitOr => true,
          BinaryOperator::BitXOR => true,
          _ => false
        }
      },
      PEMDAS::Exp => {
        match op {
          BinaryOperator::Exp => true,
          _ => false
        }
      },
      PEMDAS::MulDivMod => {
        match op {
          BinaryOperator::Mul => true,
          BinaryOperator::Div => true,
          BinaryOperator::Mod => true,
          _ => false
        }
      },
      PEMDAS::AddSub => {
        match op {
          BinaryOperator::Add => true,
          BinaryOperator::Sub => true,
          _ => false
        }
      },
      PEMDAS::Comparison => {
        match op {
          BinaryOperator::Equals => true,
          BinaryOperator::NotEquals => true,
          BinaryOperator::Greater => true,
          BinaryOperator::GreaterThanEquals => true,
          BinaryOperator::LessThan => true,
          BinaryOperator::LessThanEquals => true,
          _ => false
        }
      },
      PEMDAS::Assignation => {
        match op {
          BinaryOperator::AddAssign => true,
          BinaryOperator::SubAssign => true,
          BinaryOperator::MulAssign => true,
          BinaryOperator::DivAssign => true,
          BinaryOperator::ExpAssign => true,
          BinaryOperator::ModAssign => true,
          BinaryOperator::LogicalAndAssign => true,
          BinaryOperator::LogicalOrAssign => true,
          BinaryOperator::LogicalXORAssign => true,
          BinaryOperator::BitAndAssign => true,
          BinaryOperator::BitOrAssign => true,
          BinaryOperator::BitXORAssign => true,
          BinaryOperator::ArithmeticShrAssign => true,
          BinaryOperator::LogicalShrAssign => true,
          BinaryOperator::LogicalShlAssign => true,
          BinaryOperator::AssignPipe => true,
          BinaryOperator::Assign => true,
          _ => false
        }
      },
      PEMDAS::Pipe => {
        match op {
          BinaryOperator::Pipe => true,
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

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let mut exprs: Vec<Expression> = vec![Expression::make_expr_body(reader)?];
    let mut ops: Vec<BinaryOperator> = vec![];

    loop {
      let start_of_nth = reader.offset();

      seek::optional_whitespace(reader)?;

      let op = 'find_operator: {
        for (txt, variant) in consts::operator::BIN_MAP.into_iter() {
          if seek::begins_with(reader, txt) {
            break 'find_operator Some(variant);
          };
        };

        None
      };

      if op.is_none() {
        reader.to(start_of_nth).unwrap();

        break;
      };

      seek::optional_whitespace(reader)?;

      let Ok(expr) = Expression::make_expr_body(reader) else {
        reader.to(start_of_nth).unwrap();

        break;
      };

      exprs.push(expr);
      ops.push(op.unwrap().to_owned());
    };

    while exprs.len() > 1 {
      for state in all::<PEMDAS>() {
        'pemdas: loop {
          for i in 0..ops.len() {
            let op = &ops[i];

            if state.includes(op) {
              let a = Box::new(exprs[i].to_owned());
              let b = Box::new(exprs.remove(i + 1));

              let op = ops.remove(i);

              exprs[i] = Expression::Operator(OperatorExpressionAST {
                a, b, op, out: Type::Unresolved
              });

              continue 'pemdas;
            };
          };

          break;
        }
      };
    };

    Ok(exprs.pop().unwrap())
  }
}
