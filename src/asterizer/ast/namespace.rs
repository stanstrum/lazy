/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use crate::asterizer::{
  TokenStream,
  AsterizerError,
  MakeAst
};

use super::Structure;

#[derive(Debug)]
pub(crate) struct Namespace {
  pub name: String,
  children: HashMap<String, Structure>
}

impl MakeAst for Namespace {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    eprintln!("{}:{}: Namespace::make empty stub", file!(), line!());

    Ok(None)
  }
}
