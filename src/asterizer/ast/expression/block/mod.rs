import_export!(binding);
import_export!(control_flow);
import_export!(r#return);

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  Expression,
};

use crate::tokenizer::{
  TokenEnum,
  Punctuation,
  Grouping,
  GroupingType,
};

use crate::asterizer::error::*;

#[derive(Debug, TypeName)]
pub(crate) enum BlockChild {
  Expression(Expression),
  Binding(Binding),
  ControlFlow(ControlFlow),
  Return(Return),
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Block {
  pub(crate) children: Vec<BlockChild>,
  pub(crate) returns_last: bool,
}

impl MakeAst for BlockChild {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    #[allow(clippy::manual_map)]
    Ok({
      if let Some(binding) = stream.make()? {
        Some(Self::Binding(binding))
      } else if let Some(expression) = stream.make()? {
        Some(Self::Expression(expression))
      } else if let Some(control_flow) = stream.make()? {
        Some(Self::ControlFlow(control_flow))
      } else if let Some(r#return) = stream.make()? {
        Some(Self::Return(r#return))
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

      let Some(expr) = stream.make()? else {
        return ExpectedSnafu {
          what: "an expression",
          span: stream.span()
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
          span: stream.span()
        }.fail();
      };

      returns_last = true;
      break;
    };

    Ok(Some(Self { children, returns_last }))
  }
}
