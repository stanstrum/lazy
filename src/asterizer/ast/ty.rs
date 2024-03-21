/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use typename::TypeName;
use std::rc::Rc;

use crate::tokenizer::{
  Grouping,
  GroupingType,
  TokenEnum
};

use crate::asterizer::{
  TokenStream,
  MakeAst,
  error::*
};

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct NamedType {
  pub name: String
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct UnsizedArrayOf {
  pub ty: Rc<Type>
}

#[derive(Debug, TypeName)]
pub(crate) enum Type {
  Named(NamedType),
  UnsizedArrayOf(UnsizedArrayOf),
}

impl MakeAst for UnsizedArrayOf {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::Bracket))) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::Bracket))) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(ty) = stream.make::<Type>()?.map(Rc::new) else {
      return ExpectedSnafu {
        what: "a type",
      }.fail();
    };

    Ok(Some(Self { ty }))
  }
}

impl MakeAst for NamedType {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      return Ok(None);
    };

    let name = name.to_owned();

    Ok(Some(Self { name }))
  }
}

impl MakeAst for Type {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(named) = stream.make::<NamedType>()? {
        Some(Type::Named(named))
      } else if let Some(unsized_array_of) = stream.make::<UnsizedArrayOf>()? {
        Some(Type::UnsizedArrayOf(unsized_array_of))
      } else {
        None
      }
    })
  }
}
