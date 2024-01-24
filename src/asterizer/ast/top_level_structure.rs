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

use super::{
  Namespace,
  Function
};

#[derive(Debug)]
pub(crate) enum TopLevelStructure {
  Namespace(Namespace),
  Function(Function)
}

impl TopLevelStructure {
  pub fn name(&self) -> String {
    match self {
      TopLevelStructure::Namespace(ns) => ns.name.to_owned(),
      TopLevelStructure::Function(func) => func.decl.name.to_owned(),
    }
  }
}

impl MakeAst for TopLevelStructure {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok({
      if let Some(ns) = Namespace::make(stream)? {
        Some(TopLevelStructure::Namespace(ns))
      } else if let Some(func) = Function::make(stream)? {
        Some(TopLevelStructure::Function(func))
      } else {
        None
      }
    })
  }
}
