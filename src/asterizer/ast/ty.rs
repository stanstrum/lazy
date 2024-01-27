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

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct NamedType {
  name: String
}

#[derive(Debug)]
pub(crate) enum Type {
  Named(NamedType)
}

impl MakeAst for NamedType {
  fn make(_stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    todo!()
  }
}

impl MakeAst for Type {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(named) = stream.make::<NamedType>()? {
        Some(Type::Named(named))
      } else {
        None
      }
    })
  }
}
