mod binding;
use binding::*;

use typename::TypeName;

use crate::tokenizer::{
  TokenEnum,
  Punctuation,
  Grouping,
  GroupingType
};

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  Expression
};

use crate::asterizer::error::*;

#[derive(Debug, TypeName)]
pub(crate) enum BlockChild {
  Expression(Expression),
  Binding(Binding),
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Block {
  pub children: Vec<BlockChild>,
  pub returns_last: bool
}

impl MakeAst for BlockChild {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(expression) = stream.make::<Expression>()? {
        Some(Self::Expression(expression))
      } else if let Some(binding) = stream.make::<Binding>()? {
        // This check comes after expression because a simple expression like:
        //   `a`
        // parses for both, except is only valid as an expression.  Given that,
        // we can assume the programmer means to recall the value, not create
        // a variable with no name or type.
        Some(Self::Binding(binding))
      } else {
        None
      }
    })
  }
}

impl MakeAst for Block {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::CurlyBrace))) = stream.next_variant() else {
      return Ok(None);
    };

    let mut children = vec![];
    let mut returns_last = false;

    loop {
      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace))) = stream.peek_variant() {
        stream.seek();

        break;
      };

      let Some(expr) = stream.make::<BlockChild>()? else {
        return ExpectedSnafu {
          what: "an expression",
        }.fail();
      };

      children.push(expr);

      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Punctuation(Punctuation::Semicolon)) = stream.peek_variant() {
        stream.seek();

        // Continue parsing next expression in block, or possibly no expression
        // if the block does not return last value
        continue;
      };

      // If we're here, either a mistake has been made or this block returns its last value
      stream.skip_whitespace_and_comments();

      let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace))) = stream.next_variant() else {
        return ExpectedSnafu {
          what: "a closing curly brace",
        }.fail();
      };

      returns_last = true;
      break;
    };

    Ok(Some(Self { children, returns_last }))
  }
}
