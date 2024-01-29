/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use typename::TypeName;

use crate::tokenizer::{
  Punctuation,
  TokenEnum
};

use crate::asterizer::{
  TokenStream,
  MakeAst,
  error::*
};

use super::Type;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct FunctionDeclarationArgument {
  name: String,
  ty: Type
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct FunctionDeclarationArguments {
  args: Vec<FunctionDeclarationArgument>
}

impl MakeAst for FunctionDeclarationArgument {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      return Ok(None);
    };

    let name = name.to_owned();

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Punctuation(Punctuation::Colon)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(ty) = stream.make::<Type>()? else {
      return ExpectedSnafu {
        what: "a type",
      }.fail();
    };

    Ok(Some(Self {
      name, ty,
    }))
  }
}

impl MakeAst for FunctionDeclarationArguments {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let mut args = vec![];

    loop {
      stream.skip_whitespace_and_comments();

      let Some(arg) = stream.make::<FunctionDeclarationArgument>()? else {
        break;
      };

      args.push(arg);

      stream.skip_whitespace_and_comments();

      let Some(TokenEnum::Punctuation(Punctuation::Comma)) = stream.next_variant() else {
        break;
      };
    };

    Ok(Some(Self {
      args
    }))
  }
}
