/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use typename::TypeName;

use crate::asterizer::{
  TokenStream,
  AsterizerError
};

macro_rules! import_export {
  ($name:ident) => {
    pub(self) mod $name;
    #[allow(unused)]
    pub(crate) use $name::*;
  };

  ($name:ident, $($names:ident,)+) => {
    import_export!($name);
    import_export!($($names),+);
  };

  ($($names:ident),+) => {
    import_export!($($names,)+);
  };
}

import_export! {
  namespace,
  global_namespace,
  top_level_structure,
  structure,
  function_decl,
  function,
  type_alias,
  ty,
  expression,
  function_decl_args,
}

pub(crate) trait MakeAst where Self: Sized + TypeName {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError>;
}
