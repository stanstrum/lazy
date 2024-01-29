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

      if let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::CurlyBrace))) = stream.next_variant() {
        break;
      };

      let Some(expr) = stream.make::<Expression>()? else {
        return ExpectedSnafu {
          what: "an expression",
        }.fail();
      };

      children.push(expr);

      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Punctuation(Punctuation::Semicolon)) = stream.next_variant() {
        continue;
      };
    };

    let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace))) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "a closing curly brace"
      }.fail();
    };

    Ok(Some(Self { children }))
  }
}
