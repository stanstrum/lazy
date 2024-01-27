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
    println!("TopLevelStructure::make");

    Ok({
      if let Some(ns) = stream.make::<Namespace>()? {
        Some(TopLevelStructure::Namespace(ns))
      } else if let Some(func) = stream.make::<Function>()? {
        Some(TopLevelStructure::Function(func))
      } else {
        None
      }
    })
  }
}
