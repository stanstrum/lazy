import_export!(block);
import_export!(subexpression);
import_export!(atom);
import_export!(unary);
import_export!(binary);

use typename::TypeName;
use enum_iterator::Sequence;

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

    };

    todo!()
  }
}

#[derive(Sequence)]
enum Pemdas {
  // Parentheses -- SubExpression takes care of this,
  Exponent,
  MultiplyDivide,
  AddSubtract,
  Comparison,
  Dot,
}

impl MakeAst for Expression {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let mut resolver = ExpressionResolver::new(stream);

    if resolver.make_binary_part()?.is_none() {
      return Ok(None)
    };

    resolver.stream.push_mark();

    println!("A");
    loop {
      resolver.stream.skip_whitespace_and_comments();

      if resolver.make_binary_operator()?.is_none() {
        resolver.stream.pop_mark();
        println!("B");

        break;
      };

      println!("C");

      resolver.stream.drop_mark();
      resolver.stream.skip_whitespace_and_comments();

      if resolver.make_binary_part()?.is_none() {
        println!("D");

        return ExpectedSnafu {
          what: "an expression",
        }.fail();
      };

      println!("E");
    };

    println!("F");

    dbg!(&resolver.parts);

    let Some(combined_expr) = resolver.resolve() else {
      panic!("failed to resolve expression");
    };

    Ok(Some(combined_expr))
  }
}
