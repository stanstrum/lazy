/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use typename::TypeName;

use crate::asterizer::error::ExpectedSnafu;
use crate::asterizer::{
  AsterizerError,
  TokenStream,
  MakeAst
};

use crate::tokenizer::{
  Keyword, Operator, TokenEnum
};

use super::Type;

#[allow(unused)]
#[derive(Debug, TypeName)]
struct TypeAlias {
  ident: String,
  ty: Type
}

impl MakeAst for TypeAlias {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::Type)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Identifier(ident)) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "an identifier",
      }.fail();
    };

    let ident = ident.to_owned();

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Operator(Operator::BindAssign)) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "the binding assignment operator",
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(ty) = stream.make::<Type>()? else {
      return ExpectedSnafu {
        what: "a type",
      }.fail();
    };

    Ok(Some(Self {
      ident, ty
    }))
  }
}
