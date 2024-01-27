/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::asterizer::{
  TokenStream,
  AsterizerError,
  MakeAst
};

use crate::tokenizer::{
  Token,
  TokenEnum, Operator
};

#[derive(Debug)]
pub(crate) struct FunctionDeclaration {
  pub name: String,
}

impl MakeAst for FunctionDeclaration {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    println!("FunctionDecl::make");

    stream.push_mark();

    let Some(Token {
      token: TokenEnum::Identifier(ident),
      span: _span
    }) = stream.next() else {
      stream.pop_mark();

      return Ok(None);
    };

    let ident = ident.to_owned();

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Operator(Operator::RightArrow)) = stream.next_variant() else {
      stream.pop_mark();

      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    Ok(Some(Self {
      name: ident.to_owned(),
    }))
  }
}
