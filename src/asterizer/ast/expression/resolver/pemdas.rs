use enum_iterator::{
  Sequence,
  all,
};

use crate::asterizer::ast::{
  AsterizerError, BinaryExpression, BinaryOperator, BinaryOperatorKind, Expression, UnaryExpression, UnaryOperator, UnaryPrefixOperator, UnarySuffixOperator
};

use super::{
  ExpressionResolver,
  ExpressionPart,
};

use crate::tokenizer::{
  Span,
  GetSpan,
};

#[derive(Debug, Sequence)]
enum Pemdas {
  Dot, // and Subscript, as well as Separator
  Call,
  IncrementDecrement,
  RefDeref,
  Cast,
  Exponent,
  MultiplyDivide,
  AddSubtract,
  Logic,
  Comparison,
  Assignment,
}

impl ExpressionResolver<'_, '_> {
  pub fn resolve(mut self) -> Result<Expression, AsterizerError> {
    for pemdas in all::<Pemdas>() {
      let mut part_index = 0;

      while part_index < self.parts.len() {
        match (&pemdas, &self.parts[part_index]) {
          | (Pemdas::Exponent, ExpressionPart::Binary(BinaryOperator { kind: BinaryOperatorKind::Exponent, .. }))
          | (Pemdas::MultiplyDivide, ExpressionPart::Binary(
            | BinaryOperator { kind: BinaryOperatorKind::Multiply, .. }
            | BinaryOperator { kind: BinaryOperatorKind::Divide, .. }
          ))
          | (Pemdas::AddSubtract, ExpressionPart::Binary(
            | BinaryOperator { kind: BinaryOperatorKind::Add, .. }
            | BinaryOperator { kind: BinaryOperatorKind::Subtract, .. }
          ))
          | (Pemdas::Comparison, ExpressionPart::Binary(
            | BinaryOperator { kind: BinaryOperatorKind::Comparison, .. }
            | BinaryOperator { kind: BinaryOperatorKind::LessThan, .. }
            | BinaryOperator { kind: BinaryOperatorKind::LessThanEqual, .. }
            | BinaryOperator { kind: BinaryOperatorKind::GreaterThan, .. }
            | BinaryOperator { kind: BinaryOperatorKind::GreaterThanEqual, .. }
          ))
          | (Pemdas::Assignment, ExpressionPart::Binary(
            | BinaryOperator { kind: BinaryOperatorKind::AddAssign, .. }
            | BinaryOperator { kind: BinaryOperatorKind::SubtractAssign, .. }
            | BinaryOperator { kind: BinaryOperatorKind::MultiplyAssign, .. }
            | BinaryOperator { kind: BinaryOperatorKind::ExponentAssign, .. }
            | BinaryOperator { kind: BinaryOperatorKind::DivideAssign, .. }
            | BinaryOperator { kind: BinaryOperatorKind::ModuloAssign, .. }
            | BinaryOperator { kind: BinaryOperatorKind::Equals, .. }
            | BinaryOperator { kind: BinaryOperatorKind::BitwiseAndAssign, .. }
            | BinaryOperator { kind: BinaryOperatorKind::LogicalAndAssign, .. }
          ))
          | (Pemdas::Dot, ExpressionPart::Binary(
            | BinaryOperator { kind: BinaryOperatorKind::Dot, .. }
            | BinaryOperator { kind: BinaryOperatorKind::DerefDot, .. }
            | BinaryOperator { kind: BinaryOperatorKind::Separator, .. }
          ))
          | (Pemdas::Logic, ExpressionPart::Binary(
            | BinaryOperator { kind: BinaryOperatorKind::BitwiseAnd, .. }
            | BinaryOperator { kind: BinaryOperatorKind::LogicalAnd, .. }
          )) => {
            let lhs_index = part_index - 1;
            let rhs_index = part_index + 1;

            let ExpressionPart::Operand(rhs) = self.parts.remove(rhs_index) else {
              todo!("span impl");

              // return ExpectedSnafu {
              //   what: "an expression",
              // }.fail();
            };

            let ExpressionPart::Binary(op) = self.parts.remove(part_index) else {
              unreachable!();
            };

            let mut start_lhs_index = lhs_index;
            while !matches!(&self.parts[start_lhs_index], ExpressionPart::Operand(_)) {
              start_lhs_index -= 1;
            };

            let ExpressionPart::Operand(mut lhs) = self.parts.remove(start_lhs_index) else {
              todo!("span impl");

              // return ExpectedSnafu {
              //   what: "an expression",
              // }.fail();
            };

            while start_lhs_index < self.parts.len() && matches!(&self.parts[start_lhs_index], ExpressionPart::Unary(_)) {
              let ExpressionPart::Unary(op) = self.parts.remove(start_lhs_index) else {
                unreachable!();
              };

              let lhs_span = lhs.get_span().to_owned();

              let new_span = Span {
                end: op.get_span().end,
                ..lhs_span
              };

              lhs = Expression::Unary(
                UnaryExpression {
                  expr: Box::new(lhs),
                  op,
                  span: new_span,
                }
              );
            };

            let (lhs, rhs) = (Box::new(lhs), Box::new(rhs));

            let lhs_span = lhs.get_span().to_owned();
            let rhs_span = lhs.get_span().to_owned();

            let binary_expr = ExpressionPart::Operand(Expression::Binary(BinaryExpression {
              op,
              lhs,
              rhs,
              span: Span {
                start: lhs_span.start,
                ..rhs_span
              }
            }));

            self.parts.insert(lhs_index, binary_expr);
          },
          | (Pemdas::IncrementDecrement, ExpressionPart::Unary(
            UnaryOperator::Prefix(
              | UnaryPrefixOperator::PreIncrement
              | UnaryPrefixOperator::PreDecrement
            )
          ))
          | (Pemdas::Dot, ExpressionPart::Unary(
            UnaryOperator::Prefix(UnaryPrefixOperator::ImpliedSeparator)
          ))
          | (Pemdas::RefDeref, ExpressionPart::Unary(
            UnaryOperator::Prefix(
              | UnaryPrefixOperator::Reference
              | UnaryPrefixOperator::MutReference
            )
          )) => {
            let ExpressionPart::Unary(op) = self.parts.remove(part_index) else {
              unreachable!();
            };

            let ExpressionPart::Operand(expr) = self.parts.remove(part_index) else {
              todo!("span impl");

              // return ExpectedSnafu {
              //   what: "an expression",
              //   span: todo!("span impl")
              // }.fail();
            };

            let expr_span = expr.get_span().to_owned();
            let start = op.get_span().start;

            let unary_expr = ExpressionPart::Operand(
              Expression::Unary(UnaryExpression {
                op,
                expr: Box::new(expr),
                span: Span {
                  start,
                  ..expr_span
                }
              })
            );

            self.parts.insert(part_index, unary_expr);

            part_index += 1;
          },
          | (Pemdas::IncrementDecrement, ExpressionPart::Unary(
            UnaryOperator::Suffix(
              | UnarySuffixOperator::PostIncrement
              | UnarySuffixOperator::PostDecrement
              | UnarySuffixOperator::Call { .. }
            )
          ))
          | (Pemdas::Dot, ExpressionPart::Unary(
            UnaryOperator::Suffix(
              UnarySuffixOperator::Subscript { .. }
            )
          ))
          | (Pemdas::Cast, ExpressionPart::Unary(
            UnaryOperator::Suffix(
              UnarySuffixOperator::Cast { .. }
            )
          )) => {
            let ExpressionPart::Unary(op) = self.parts.remove(part_index) else {
              unreachable!();
            };

            let ExpressionPart::Operand(expr) = self.parts.remove(part_index - 1) else {
              todo!("span impl");

              // return ExpectedSnafu {
              //   what: "an expression",
              // }.fail();
            };

            let expr_span = expr.get_span().to_owned();
            let end = op.get_span().end;

            let unary_expr = ExpressionPart::Operand(
              Expression::Unary(UnaryExpression {
                op,
                expr: Box::new(expr),
                span: Span {
                  end,
                  ..expr_span
                }
              })
            );

            self.parts.insert(part_index - 1, unary_expr);
          },
          _ => {
            part_index += 1;
          }
        }
      };
    };

    let len = self.parts.len();
    if len != 1 {
      // dbg!(&self.parts);

      panic!("pemdas failed: {len} parts");
    };

    let ExpressionPart::Operand(combined_expr) = self.parts.pop().unwrap() else {
      panic!("pemdas failed: last part is not an operand");
    };

    Ok(combined_expr)
  }
}
