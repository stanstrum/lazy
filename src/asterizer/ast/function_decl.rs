/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::asterizer::ast::FunctionDeclarationArguments;
use crate::asterizer::{
  TokenStream,
  AsterizerError,
  MakeAst
};

use crate::tokenizer::{
  Token,
  TokenEnum,
  Operator,
  Punctuation
};

use super::Type;

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct FunctionDeclaration {
  pub name: String,
  return_type: Option<Type>,
  args: Option<FunctionDeclarationArguments>,
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

    let name = ident.to_owned();

    stream.skip_whitespace_and_comments();

    stream.push_mark();

    let return_type = {
      if let Some(TokenEnum::Operator(Operator::RightArrow)) = stream.next_variant() {
        stream.skip_whitespace_and_comments();

        if let Some(ty) = Type::make(stream)? {
          stream.drop_mark();

          Some(ty)
        } else {
          stream.pop_mark();

          None
        }
      } else {
        stream.pop_mark();

        None
      }
    };

    stream.skip_whitespace_and_comments();

    stream.push_mark();

    let args = {
      if let Some(TokenEnum::Punctuation(Punctuation::Colon)) = stream.next_variant() {
        match FunctionDeclarationArguments::make(stream)? {
          Some(args) => {
            stream.drop_mark();

            Some(args)
          },
          None => {
            stream.pop_mark();

            None
          }
        }
      } else {
        stream.pop_mark();

        None
      }
    };

    stream.drop_mark();

    Ok(Some(Self {
      name, return_type, args
    }))
  }
}
