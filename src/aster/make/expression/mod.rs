/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::{
  SourceReader,
  AsterResult,
  ast::*,
  errors::*,
  seek,
  consts
};

use crate::{
  try_make,
  intent
};

mod atom;
mod block;
mod sub;
mod control_flow;
mod binding;

use enum_iterator::{Sequence, all};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Sequence)]
enum PEMDAS {
  DotAndSubscriptAndCall,
  Unary,
  Bit,
  Exp,
  MulDivMod,
  AddSub,
  Comparison,
  Testing,
  Assignation,
  Pipe,
}

impl PEMDAS {
  fn includes(&self, op: &Operator) -> bool {
    match self {
      PEMDAS::DotAndSubscriptAndCall => {
        matches!(op,
          | Operator::Binary(BinaryOperator::Dot)
          | Operator::Binary(BinaryOperator::DerefDot)
          | Operator::UnarySfx(UnarySfxOperator::Subscript { .. })
          | Operator::UnarySfx(UnarySfxOperator::Call { .. })
        )
      },
      PEMDAS::Unary => {
        matches!(op,
          | Operator::UnarySfx(UnarySfxOperator::PostIncrement)
          | Operator::UnarySfx(UnarySfxOperator::PostDecrement)
          | Operator::UnarySfx(UnarySfxOperator::Cast { .. })
          | Operator::UnaryPfx(UnaryPfxOperator::MutRef)
          | Operator::UnaryPfx(UnaryPfxOperator::Ref)
          | Operator::UnaryPfx(UnaryPfxOperator::Deref)
          | Operator::UnaryPfx(UnaryPfxOperator::Not)
          | Operator::UnaryPfx(UnaryPfxOperator::Neg)
          | Operator::UnaryPfx(UnaryPfxOperator::NotNeg)
          | Operator::UnaryPfx(UnaryPfxOperator::PreIncrement)
          | Operator::UnaryPfx(UnaryPfxOperator::PreDecrement)
        )
      },
      PEMDAS::Bit => {
        matches!(op,
          | Operator::Binary(BinaryOperator::BitAnd)
          | Operator::Binary(BinaryOperator::BitOr)
          | Operator::Binary(BinaryOperator::BitXOR)
        )
      },
      PEMDAS::Exp => {
        matches!(op,
          | Operator::Binary(BinaryOperator::Exp)
        )
      },
      PEMDAS::MulDivMod => {
        matches!(op,
          | Operator::Binary(BinaryOperator::Mul)
          | Operator::Binary(BinaryOperator::Div)
          | Operator::Binary(BinaryOperator::Mod)
        )
      },
      PEMDAS::AddSub => {
        matches!(op,
          | Operator::Binary(BinaryOperator::Add)
          | Operator::Binary(BinaryOperator::Sub)
        )
      },
      PEMDAS::Comparison => {
        matches!(op,
          | Operator::Binary(BinaryOperator::Equals)
          | Operator::Binary(BinaryOperator::NotEquals)
          | Operator::Binary(BinaryOperator::Greater)
          | Operator::Binary(BinaryOperator::GreaterThanEquals)
          | Operator::Binary(BinaryOperator::LessThan)
          | Operator::Binary(BinaryOperator::LessThanEquals)
        )
      },
      PEMDAS::Testing => {
        matches!(op,
          | Operator::Binary(BinaryOperator::LogicalAnd)
          | Operator::Binary(BinaryOperator::LogicalOr)
          | Operator::Binary(BinaryOperator::LogicalXOR)
        )
      },
      PEMDAS::Assignation => {
        matches!(op,
          | Operator::Binary(BinaryOperator::AddAssign)
          | Operator::Binary(BinaryOperator::SubAssign)
          | Operator::Binary(BinaryOperator::MulAssign)
          | Operator::Binary(BinaryOperator::DivAssign)
          | Operator::Binary(BinaryOperator::ExpAssign)
          | Operator::Binary(BinaryOperator::ModAssign)
          | Operator::Binary(BinaryOperator::LogicalAndAssign)
          | Operator::Binary(BinaryOperator::LogicalOrAssign)
          | Operator::Binary(BinaryOperator::LogicalXORAssign)
          | Operator::Binary(BinaryOperator::BitAndAssign)
          | Operator::Binary(BinaryOperator::BitOrAssign)
          | Operator::Binary(BinaryOperator::BitXORAssign)
          | Operator::Binary(BinaryOperator::ArithmeticShrAssign)
          | Operator::Binary(BinaryOperator::LogicalShrAssign)
          | Operator::Binary(BinaryOperator::LogicalShlAssign)
          | Operator::Binary(BinaryOperator::AssignPipe)
          | Operator::Binary(BinaryOperator::Assign)
        )
      },
      PEMDAS::Pipe => {
        matches!(op,
          | Operator::Binary(BinaryOperator::Pipe)
        )
      },
    }
  }
}

enum LastExprComponent {
  Empty,
  PfxOperator,
  SfxOperator,
  Body,
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
        offset: reader.offset(),
        path: reader.path.clone()
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

      let Ok(expr) = intent!(Expression::make_expr_body, reader) else {
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
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail()
    }
  }

  pub fn make_unary_pfx(reader: &mut SourceReader) -> AsterResult<UnaryPfxOperator> {
    let start = reader.offset();

    let result = 'result: {
      for (txt, variant) in consts::operator::UNARY_PFX_MAP.into_iter() {
        if seek::begins_with(reader, txt) {
          if matches!(variant, UnaryPfxOperator::MutRef) && seek::optional_whitespace(reader)? == 0 {
            reader.to(start).unwrap();

            continue;
          };

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
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail()
    }
  }

  fn make_cast(reader: &mut SourceReader) -> AsterResult<UnarySfxOperator> {
    let start = reader.offset();

    let result = 'result: {
      if !seek::begins_with(reader, "as") {
        break 'result None;
      };

      if seek::required_whitespace(reader).is_err() {
        break 'result None;
      };

      let Ok(ty) = intent!(TypeAST::make, reader) else {
        break 'result None;
      };

      Some(UnarySfxOperator::Cast { to: ty, method: None })
    };

    if let Some(result) = result {
      Ok(result)
    } else {
      reader.to(start).unwrap();

      ExpectedSnafu {
        what: "Unary Prefix Operator",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail()
    }
  }

  pub fn make_unary_sfx(reader: &mut SourceReader) -> AsterResult<UnarySfxOperator> {
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

      if let Ok(fn_call) = Expression::make_fn_call(reader) {
        Ok(fn_call)
      } else if let Ok(subscript) = Expression::make_subscript(reader) {
        Ok(subscript)
      } else if let Ok(cast) = Expression::make_cast(reader) {
        Ok(cast)
      } else {
        ExpectedSnafu {
          what: "Unary Prefix Operator",
          offset: reader.offset(),
          path: reader.path.clone()
        }.fail()
      }
    }
  }

  fn make_fn_call(reader: &mut SourceReader) -> AsterResult<UnarySfxOperator> {
    let start = reader.offset();

    let result = 'result: {
      let mut args: Vec<Expression> = vec![];

      if !seek::begins_with(reader, consts::grouping::OPEN_PARENTHESIS) {
        break 'result None;
      };

      loop {
        seek::optional_whitespace(reader)?;

        if seek::begins_with(reader, consts::grouping::CLOSE_PARENTHESIS) {
          break;
        };

        let Ok(arg_expr) = Expression::make(reader) else {
          break 'result None;
        };

        args.push(arg_expr);

        seek::optional_whitespace(reader)?;

        if !seek::begins_with(reader, consts::punctuation::COMMA) {
          if !seek::begins_with(reader, consts::grouping::CLOSE_PARENTHESIS) {
            return reader.set_intent(
              ExpectedSnafu {
                what: "Close Parenthesis",
                offset: reader.offset(),
                path: reader.path.clone()
              }.fail()
            );
          } else {
            break;
          };
        };
      };

      Some(UnarySfxOperator::Call { args })
    };

    if let Some(result) = result {
      Ok(result)
    } else {
      reader.to(start).unwrap();

      ExpectedSnafu {
        what: "Unary Prefix Operator",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail()
    }
  }

  fn make_subscript(reader: &mut SourceReader) -> AsterResult<UnarySfxOperator> {
    let start = reader.offset();

    let result = 'result: {
      if !seek::begins_with(reader, consts::grouping::OPEN_BRACKET) {
        break 'result None;
      };

      seek::optional_whitespace(reader)?;

      let Ok(arg) = intent!(Expression::make, reader) else {
        break 'result None;
      };

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::grouping::CLOSE_BRACKET) {
        reader.set_intent(
          ExpectedSnafu {
            what: "Close Bracket",
            offset: reader.offset(),
            path: reader.path.clone(),
          }.fail::<()>()
        ).unwrap_err();

        break 'result None;
      };

      Some(UnarySfxOperator::Subscript {
        arg: Box::new(arg), dest: false
      })
    };

    if let Some(result) = result {
      Ok(result)
    } else {
      reader.to(start).unwrap();

      ExpectedSnafu {
        what: "Unary Prefix Operator",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail()
    }
  }

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let mut exprs: Vec<Expression> = vec![];
    let mut ops: Vec<Operator> = vec![];

    let mut last = LastExprComponent::Empty;

    loop {
      match last {
        LastExprComponent::Empty => {
          if let Some(expr) = try_make!(Expression::make_expr_body, reader) {
            exprs.push(expr);

            last = LastExprComponent::Body;
          } else if let Ok(pfx) = Expression::make_unary_pfx(reader) {
            ops.push(Operator::UnaryPfx(pfx));

            last = LastExprComponent::PfxOperator;
          } else {
            return ExpectedSnafu {
              what: "Expression",
              offset: reader.offset(),
              path: reader.path.clone()
            }.fail();
          };
        },
        LastExprComponent::PfxOperator => {
          seek::optional_whitespace(reader)?;

          if let Some(expr) = try_make!(Expression::make_expr_body, reader) {
            exprs.push(expr);

            last = LastExprComponent::Body;
          } else if let Ok(pfx) = Expression::make_unary_pfx(reader) {
            ops.push(Operator::UnaryPfx(pfx));

            last = LastExprComponent::PfxOperator;
          } else {
            return reader.set_intent(
              ExpectedSnafu {
                what: "Expression",
                offset: reader.offset(),
                path: reader.path.clone()
              }.fail()
            );
          };
        },
        LastExprComponent::Body | LastExprComponent::SfxOperator => {
          let body_start = reader.offset();
          seek::optional_whitespace(reader)?;

          if let Ok((op, expr)) = Expression::make_binary_half(reader) {
            ops.push(Operator::Binary(op));
            exprs.push(expr);

            last = LastExprComponent::Body;
          } else if let Ok(sfx) = Expression::make_unary_sfx(reader) {
            ops.push(Operator::UnarySfx(sfx));

            last = LastExprComponent::SfxOperator;
          } else {
            reader.to(body_start).unwrap();

            break;
          };
        },
      };
    };

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
              Operator::UnaryPfx(_) => {
                let expr = Box::new(exprs[i].to_owned());
                let Operator::UnaryPfx(op) = ops.remove(i) else {
                  unreachable!();
                };

                let new_expr = Expression::UnaryOperator(UnaryOperatorExpressionAST {
                  span: expr.span(),
                  out: Type::Unresolved,
                  expr, op: UnaryOperator::UnaryPfx(op)
                });

                exprs[i] = new_expr;

                continue 'pemdas;
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

                continue 'pemdas;
              },
            }
          };
        };

        break;
      }
    };

    if exprs.len() != 1 || !ops.is_empty() {
      dbg!(&exprs);
      dbg!(&ops);

      panic!("PEMDAS failed");
    };

    Ok(exprs.pop().unwrap())
  }
}
