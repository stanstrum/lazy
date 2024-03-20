/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use typename::TypeName;

use crate::tokenizer::{
  TokenEnum,
  Punctuation,
  Grouping,
  GroupingType
};

use crate::asterizer::ast::{
  AsterizerError,
  TokenStream,
  MakeAst
};

use crate::asterizer::error::ExpectedSnafu;

use super::Expression;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct BlockExpression {
  pub children: Vec<Expression>
}

impl MakeAst for BlockExpression {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::CurlyBrace))) = stream.next_variant() else {
      return Ok(None);
    };

    let mut children = vec![];

    loop {
      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace))) = stream.peek_variant() {
        stream.seek();

        break;
      };

      let Some(expr) = stream.make::<Expression>()? else {
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

      // TODO: mark this block as returning its last value like in Rust
      break;
    };

    Ok(Some(Self { children }))
  }
}
