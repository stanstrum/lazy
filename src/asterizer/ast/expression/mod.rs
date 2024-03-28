import_export!(block);
import_export!(subexpression);
import_export!(atom);
import_export!(unary);
import_export!(binary);

use typename::TypeName;
use enum_iterator::{
  Sequence,
  all,
};

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
};

use crate::asterizer::error::ExpectedSnafu;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) enum Expression {
  Atom(Atom),
  Block(Block),
  SubExpression(SubExpression),
  Unary(UnaryExpression),
  Binary(BinaryExpression),
}

#[derive(Debug, TypeName)]
enum NonOperatorExpression {
  Atom(Atom),
  Block(Block),
  SubExpression(SubExpression),
}

impl MakeAst for NonOperatorExpression {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(block) = stream.make()? {
        Some(Self::Block(block))
      } else if let Some(subexpr) = stream.make()? {
        Some(Self::SubExpression(subexpr))
      } else if let Some(atom) = stream.make()? {
        Some(Self::Atom(atom))
      } else {
        None
      }
    })
  }
}

impl From<NonOperatorExpression> for Expression {
  fn from(value: NonOperatorExpression) -> Self {
    match value {
      NonOperatorExpression::Atom(atom) => Expression::Atom(atom),
      NonOperatorExpression::Block(block) => Expression::Block(block),
      NonOperatorExpression::SubExpression(sub) => Expression::SubExpression(sub),
    }
  }
}

#[derive(Debug)]
enum ExpressionPart {
  Unary(UnaryOperator),
  Binary(BinaryOperator),
  Operand(Expression),
}

struct ExpressionResolver<'a> {
  pub stream: &'a mut TokenStream,
  parts: Vec<ExpressionPart>
}

impl<'a> ExpressionResolver<'a> {
  pub fn new(stream: &'a mut TokenStream) -> Self {
    Self {
      stream,
      parts: vec![],
    }
  }

  pub fn make_binary_operator(&mut self) -> Result<Option<()>, AsterizerError> {
    let Some(op) = self.stream.make::<BinaryOperator>()? else {
      return Ok(None);
    };

    self.parts.push(ExpressionPart::Binary(op));

    Ok(Some(()))
  }

  pub fn make_binary_part(&mut self) -> Result<Option<()>, AsterizerError> {
    loop {
      let Some(op) = self.stream.make::<UnaryPrefixOperator>()? else {
        break;
      };

      self.parts.push(ExpressionPart::Unary(UnaryOperator::Prefix(op)));

      self.stream.skip_whitespace_and_comments();
    };

    let Some(expr) = self.stream.make::<NonOperatorExpression>()? else {
      return Ok(None);
    };

    self.parts.push(ExpressionPart::Operand(expr.into()));

    loop {
      self.stream.push_mark();
      self.stream.skip_whitespace_and_comments();

      let Some(op) = self.stream.make::<UnarySuffixOperator>()? else {
        self.stream.pop_mark();
        break;
      };

      self.stream.drop_mark();

      self.parts.push(ExpressionPart::Unary(UnaryOperator::Suffix(op)));
    };

    Ok(Some(()))
  }

  pub fn resolve(&mut self) -> Option<Expression> {
    while self.parts.len() > 1 {
      let before = self.parts.len();

      'pemdas: for pemdas in all::<Pemdas>() {
        let mut did_work = false;

        'repeat_find_operator: loop {
          'find_operator: for i in 0.. {
            dbg!(&pemdas, &self.parts);

            if i >= self.parts.len() {
              if did_work {
                did_work = false;

                continue 'repeat_find_operator;
              } else {
                continue 'pemdas;
              };
            };

            let ExpressionPart::Binary(operator_candidate) = &self.parts[i] else {
              continue 'find_operator;
            };

            match (&pemdas, operator_candidate) {
              | (Pemdas::Exponent, BinaryOperator::Exponent)
              | (Pemdas::MultiplyDivide,
                | BinaryOperator::Multiply
                | BinaryOperator::Divide
              )
              | (Pemdas::AddSubtract,
                | BinaryOperator::Add
                | BinaryOperator::Subtract
              )
              | (Pemdas::Comparison, BinaryOperator::Comparison)
              | (Pemdas::Assignment, BinaryOperator::Equals)
              | (Pemdas::Dot,
                | BinaryOperator::Dot
                | BinaryOperator::DerefDot
              ) => {},
              _ => continue 'find_operator
            };

            let lhs_index = 'lhs: {
              for lhs_index in (0..i).rev() {
                if matches!(self.parts[lhs_index], ExpressionPart::Operand(_)) {
                  break 'lhs Some(lhs_index);
                }
              }

              None
            };

            let rhs_index = 'rhs: {
              for rhs_index in i..self.parts.len() {
                if matches!(self.parts[rhs_index], ExpressionPart::Operand(_)) {
                  break 'rhs Some(rhs_index);
                }
              }

              None
            };

            assert!(lhs_index.is_some(), "found no left-hand-side");
            assert!(rhs_index.is_some(), "found no right-hand-side");

            let starting_point = lhs_index.unwrap();

            let rhs = self.parts.remove(rhs_index.unwrap());
            let op = self.parts.remove(i);
            let lhs = self.parts.remove(lhs_index.unwrap());

            let (
              ExpressionPart::Operand(lhs),
              ExpressionPart::Binary(op),
              ExpressionPart::Operand(rhs),
            ) = (lhs, op, rhs) else {
              unreachable!();
            };

            let (lhs, rhs) = (Box::new(lhs), Box::new(rhs));
            let bin_expr = Expression::Binary(BinaryExpression { op, lhs, rhs });

            self.parts.insert(starting_point, ExpressionPart::Operand(bin_expr));

            did_work = true;
          };
        };
      };

      let after = self.parts.len();

      assert!(before != after, "no work done");
    };

    if let Some(ExpressionPart::Operand(expr)) = self.parts.pop() {
      Some(expr)
    } else {
      None
    }
  }
}

#[derive(Debug, Sequence)]
enum Pemdas {
  // Parentheses -- SubExpression takes care of this,
  Exponent,
  MultiplyDivide,
  AddSubtract,
  Comparison,
  Assignment,
  Dot,
}

impl MakeAst for Expression {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let mut resolver = ExpressionResolver::new(stream);

    if resolver.make_binary_part()?.is_none() {
      return Ok(None)
    };

    loop {
      resolver.stream.push_mark();
      resolver.stream.skip_whitespace_and_comments();

      if resolver.make_binary_operator()?.is_none() {
        resolver.stream.pop_mark();

        break;
      };

      resolver.stream.drop_mark();
      resolver.stream.skip_whitespace_and_comments();

      if resolver.make_binary_part()?.is_none() {
        dbg!(&resolver.parts);

        return ExpectedSnafu {
          what: "an expression",
        }.fail();
      };
    };

    dbg!(&resolver.parts);

    let Some(combined_expr) = resolver.resolve() else {
      panic!("failed to resolve expression");
    };

    Ok(Some(combined_expr))
  }
}
