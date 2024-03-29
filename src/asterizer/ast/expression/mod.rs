import_export!(block);
import_export!(subexpression);
import_export!(atom);
import_export!(unary);
import_export!(binary);

mod resolver;
use resolver::ExpressionResolver;

use typename::TypeName;

use crate::asterizer::ast::{
  AsterizerError,
  TokenStream,
  MakeAst,
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
pub(super) enum NonOperatorExpression {
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

    let combined_expr = resolver.resolve()?;

    Ok(Some(combined_expr))
  }
}
